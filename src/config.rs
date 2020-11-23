use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub webdriver_url: String,
}
