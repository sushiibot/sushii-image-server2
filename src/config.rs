use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub webdriver_url: String,

    #[serde(default)]
    pub pool_keep_alive: bool,
    pub pool_size: Option<usize>,
}
