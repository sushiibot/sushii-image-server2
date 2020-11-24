use anyhow::{Error, Result};
use async_trait::async_trait;
use fantoccini::Client;
/*
use deadpool::managed::{self, Manager, RecycleResult, RecycleError};

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
        let client = Client::new(&self.url).await?;

        // Browser windows/tabs will NOT be closed with client.persist(), which
        // is good so that sessions can be recycled. HOWEVER this causes a
        // memory leak in the ChromeDriver Docker image as when
        // sushii-image-server exits, windows stay open and will accumulate.
        // client.persist().await?;

        Ok(client)
    }

    async fn recycle(&self, conn: &mut Client) -> RecycleResult<Error> {
        // Instead of keeping clients open, they are terminated which also
        // closes the associated browser process. This prevents memory leaks as
        // mentioned above and prevents browsers from accumulating memory use.

        /*
        conn.close().await
            .map_err(Error::new)
            .map_err(RecycleError::from)?;
        */

        None
    }
}
*/

use std::ops::{Deref, DerefMut};
use tokio::sync::Semaphore;

/// This Browser "Pool" isn't really an actual pool in the sense that it keeps
/// connections and browser processes running. This will drop the fantoccini
/// client when a BrowserClientGuard goes out of scope in order to have the sole
/// purpose of limiting the max number of running browser processes and
/// WebDriver clients running at a given time
pub struct BrowserPool {
    semaphore: Semaphore,
    /// Initial number of semaphore permits
    permits: usize,
    url: String,
}

#[async_trait]
pub trait Manager {
    async fn get(&self) -> Result<BrowserClientGuard<'_>>;
}

#[async_trait]
impl Manager for BrowserPool {
    async fn get(&self) -> Result<BrowserClientGuard<'_>> {
        Ok(BrowserClientGuard {
            _permit: self.semaphore.acquire().await,
            client: Client::new(&self.url).await?,
        })
    }
}

impl BrowserPool {
    pub fn new<S: Into<String>>(url: S, permits: usize) -> Self {
        Self {
            semaphore: Semaphore::new(permits),
            permits: permits,
            url: url.into(),
        }
    }

    pub fn permits(&self) -> usize {
        self.permits
    }

    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    pub fn status(&self) -> BrowserPoolStatus {
        BrowserPoolStatus {
            permits: self.permits(),
            available_permits: self.available_permits(),
        }
    }
}

/// Browser client from pool
pub struct BrowserClientGuard<'a> {
    _permit: tokio::sync::SemaphorePermit<'a>,
    client: Client,
}

impl Deref for BrowserClientGuard<'_> {
    type Target = Client;

    fn deref(&self) -> &Client {
        &self.client
    }
}

impl DerefMut for BrowserClientGuard<'_> {
    fn deref_mut(&mut self) -> &mut Client {
        &mut self.client
    }
}

#[derive(Debug)]
pub struct BrowserPoolStatus {
    pub permits: usize,
    pub available_permits: usize,
}
