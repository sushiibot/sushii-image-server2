use anyhow::{Error, Result};
use async_trait::async_trait;
use deadpool::managed::{self, Manager, RecycleResult};
use fantoccini::Client;

pub type Pool = managed::Pool<Client, Error>;

pub struct BrowserManager {
    url: String,
}

impl BrowserManager {
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self { url: url.into() }
    }
}

#[async_trait]
impl Manager<Client, Error> for BrowserManager {
    async fn create(&self) -> Result<Client> {
        let mut client = Client::new(&self.url).await?;

        // Browser windows/tabs will NOT be closed, which is good so that
        // sessions can be recycled. HOWEVER this causes a memory leak in the
        // ChromeDriver Docker image as when sushii-image-server exits, windows
        // stay open and will accumulate.
        client.persist().await?;

        Ok(client)
    }
    async fn recycle(&self, _conn: &mut Client) -> RecycleResult<Error> {
        Ok(())
    }
}

