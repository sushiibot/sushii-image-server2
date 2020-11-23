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
use rocket_contrib::json::Json;
use rocket_contrib::json::JsonValue;
use serde::Deserialize;
use std::result::Result as StdResult;
use rocket_prometheus::PrometheusMetrics;

mod browser_pool;
mod config;

use browser_pool::{BrowserManager, Pool};
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
    pool: State<'_, Pool>,
) -> StdResult<Content<Vec<u8>>, Debug<Error>> {
    tracing::info!("Pool status: {:#?}", pool.status());

    let mut conn = pool.get().await.unwrap();

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

    let mgr = BrowserManager::new(&config.webdriver_url);
    let pool = Pool::new(mgr, 4);

    let prometheus = PrometheusMetrics::new();

    let r = rocket::custom(figment)
        .manage(pool)
        .manage(handlebars)
        .attach(prometheus.clone())
        .mount("/", routes![index, template]);

    Ok(r)
}

#[rocket::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    rocket()?.launch().await?;

    Ok(())
}
