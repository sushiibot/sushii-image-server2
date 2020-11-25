#[cfg(test)]
mod tests;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use anyhow::{Error, Result};
use deadpool::unmanaged::{Object, Pool};
use fantoccini::Client;
use handlebars::Handlebars;
use rocket::fairing::AdHoc;
use rocket::http::ContentType;
use rocket::response::content::Content;
use rocket::response::Debug;
use rocket::State;
use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::serve::StaticFiles;
use rocket_prometheus::PrometheusMetrics;
use serde::Deserialize;
use std::result::Result as StdResult;
use tokio::signal::unix::{signal, SignalKind};

mod config;

use config::Config;

#[derive(Deserialize)]
struct TemplateContext {
    /// Template name
    pub name: String,
    /// Screenshot width
    pub width: Option<u64>,
    /// Screenshot height
    pub height: Option<u64>,
    /// Jpeg image quality (0-100)
    pub jpeg_quality: Option<u64>,
    /// Template context data
    pub ctx: JsonValue,
}

#[post("/template", data = "<template_ctx>")]
async fn template(
    template_ctx: Json<TemplateContext>,
    hbs: State<'_, Handlebars<'_>>,
    pool: State<'_, Pool<Option<Client>>>,
    config: State<'_, Config>,
) -> StdResult<Content<Vec<u8>>, Debug<Error>> {
    let status = pool.status();
    tracing::info!("Pool status: {:?}", status);

    let mut conn = if let Some(client) = Object::take(pool.get().await) {
        client
    } else {
        Client::new(&config.webdriver_url)
            .await
            .map_err(Error::new)?
    };

    let html = hbs
        .render(&template_ctx.name, &template_ctx.ctx)
        .map(base64::encode)
        .map_err(Error::new)
        .map_err(Debug)?;

    conn.goto(&format!("data:text/html;base64,{}", html))
        .await
        .map_err(Error::new)
        .map_err(Debug)?;

    let session_id = conn.session_id().await;
    tracing::info!("session ID: {:?}", session_id);

    // client can only have 1 at a time??
    let bytes = conn.screenshot().await.map_err(Error::new).map_err(Debug)?;

    if !config.pool_keep_alive {
        if let Err(_) = pool.try_add(None) {
            tracing::warn!("Failed to add None to pool");
        }
    } else {
        if let Err(_) = pool.try_add(Some(conn)) {
            tracing::warn!("Failed to add Some(conn) to pool");
        }
    }

    Ok(Content(ContentType::PNG, bytes))
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn rocket() -> Result<rocket::Rocket> {
    let figment = rocket::Config::figment();
    let config: Config = figment.extract()?;

    let mut handlebars = Handlebars::new();
    handlebars.register_templates_directory(".hbs", "templates/")?;

    let pool_size = config.pool_size.unwrap_or(4);
    let pool_init: Vec<Option<Client>> = vec![None; pool_size];
    let pool = Pool::from(pool_init);

    tracing::info!("Config: {:#?}", &config);
    tracing::info!("Using WebDriver URL: {}", &config.webdriver_url);

    let prometheus = PrometheusMetrics::new();

    let r = rocket::custom(figment)
        .manage(pool.clone())
        .manage(handlebars)
        .mount("/static", StaticFiles::from("./static"))
        .attach(prometheus.clone())
        .attach(AdHoc::config::<Config>())
        .mount("/", routes![index, template]);

    let handle = r.shutdown();

    let signal_kinds = vec![
        SignalKind::hangup(),
        SignalKind::interrupt(),
        SignalKind::terminate(),
    ];

    for signal_kind in signal_kinds {
        let mut stream = signal(signal_kind).unwrap();
        let handle = handle.clone();
        let pool = pool.clone();

        tokio::spawn(async move {
            stream.recv().await;
            tracing::info!("Signal received, shutting down...");
            handle.shutdown();

            let status = pool.status();
            tracing::info!("Closing browser pool... ({} objects)", status.size);
            for _ in 0..status.size {
                pool.remove().await;
            }

            tracing::info!("bye");
        });
    }

    Ok(r)
}

#[rocket::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    let r = rocket()?;

    // Start server
    r.launch().await?;

    Ok(())
}
