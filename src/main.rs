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
    let mut conn = pool.get().await.unwrap();

    let html = hbs
        .render(&template_ctx.name, &template_ctx.ctx)
        .map_err(Error::new)
        .map_err(Debug)?;

    conn.goto(&format!("data:text/html;charset=utf-8,{}", html))
        .await
        .map_err(Error::new)
        .map_err(Debug)?;

    // client can only have 1 at a time??
    let bytes = conn.screenshot().await.map_err(Error::new).map_err(Debug)?;

    Ok(Content(ContentType::PNG, bytes))
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() -> Result<()> {
    let figment = rocket::Config::figment();

    let config: Config = figment.extract()?;

    // let webdriver_client = WebDriverClient::new(&config.webdriver_url).await?;

    let mut handlebars = Handlebars::new();
    handlebars.register_templates_directory(".hbs", "templates/")?;

    let mgr = BrowserManager::new(&config.webdriver_url);
    let pool = Pool::new(mgr, 4);

    rocket::custom(figment)
        .manage(pool)
        .manage(handlebars)
        .mount("/", routes![index, template])
        .launch()
        .await?;

    Ok(())
}
