#[cfg(test)]
#[macro_use] extern crate bencher;

#[cfg(test)]
mod tests;

#[macro_use]
extern crate rocket;

use anyhow::{Error, Result};
use handlebars::Handlebars;
use rocket::http::ContentType;
use rocket::response::content::Content;
use rocket::response::Debug;
use rocket::State;
use rocket::fairing::AdHoc;
use rocket_contrib::json::Json;
use rocket_contrib::json::JsonValue;
use serde::Deserialize;
use std::result::Result as StdResult;
use rocket_prometheus::PrometheusMetrics;
use tokio::signal::unix::{signal, SignalKind};
use fantoccini::Client;

mod browser_pool;
mod config;

use deadpool::unmanaged::{Pool, Object};

// use browser_pool::{BrowserPoolManager, BrowserPool, Pool, PersistedBrowserPool, BrowserManager};
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

    let mut conn = if let Some(client) = Object::take(pool.get().await) {
        client
    } else {
        Client::new(&config.webdriver_url).await.map_err(Error::new)?
    };

    let html = hbs
        .render(&template_ctx.name, &template_ctx.ctx)
        .map_err(Error::new)
        .map_err(Debug)?;

    conn.goto(&format!("data:text/html;charset=utf-8,{}", html))
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

    let pool_init: Vec<Option<Client>> = (0..pool_size).map(|_| None).collect();

    let pool = Pool::from(pool_init);

    // Select pool type
    /*
    let pool: Pool = match config.pool_type.map_or("ephemeral", |t| t.as_str()) {
        "persisted" => {
            let mgr = BrowserManager::new(&config.webdriver_url);
            let pool = PersistedBrowserPool::new(mgr, pool_size);

            pool.into()
        },
        "ephemeral" => {
            BrowserPool::new(&config.webdriver_url, pool_size).into()
        }
        _ => {
            anyhow::bail!("Invalid browser pool type, requires either `persisted`, `ephemeral`");
        }
    };
    */

    tracing::info!("Config: {:#?}", &config);
    tracing::info!("Connecting to WebDriver URL: {}", &config.webdriver_url);

    let prometheus = PrometheusMetrics::new();

    let r = rocket::custom(figment)
        .manage(pool)
        .manage(handlebars)
        .attach(prometheus.clone())
        .attach(AdHoc::config::<Config>())
        .mount("/", routes![index, template]);

    Ok(r)
}

#[rocket::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    let r = rocket()?;

    /*
    let handle = r.shutdown();

    let signal_kinds = vec![
        SignalKind::hangup(),
        SignalKind::interrupt(),
        SignalKind::terminate(),
    ];

    for signal_kind in signal_kinds {
        let mut stream = signal(signal_kind).unwrap();
        let shard_manager = client.shard_manager.clone();
        let pool = pool.clone();
        let mut metrics_sender = metrics_sender.clone();

        tokio::spawn(async move {
            stream.recv().await;
            tracing::info!("Signal received, shutting down...");
            shard_manager.lock().await.shutdown_all().await;

            tracing::info!("Closing database pool...");
            pool.close().await;

            tracing::info!("Shutting down metrics server...");
            metrics_sender
                .send(())
                .await
                .expect("Failed to shut down metrics server");

            tracing::info!("bye");
        });
    }
    */

    // Start server
    r.launch().await?;

    Ok(())
}
