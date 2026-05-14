//! Shared HTTP client.  Single instance per process, modest timeouts, gzip,
//! no cookies.  We pass the client explicitly so tests can substitute one.

use crate::error::AppResult;
use once_cell::sync::OnceCell;
use std::time::Duration;

static CLIENT: OnceCell<reqwest::Client> = OnceCell::new();

const UA: &str = concat!("JobSearchTool/", env!("CARGO_PKG_VERSION"));

pub fn client() -> AppResult<&'static reqwest::Client> {
    if let Some(c) = CLIENT.get() {
        return Ok(c);
    }
    let c = reqwest::Client::builder()
        .user_agent(UA)
        .timeout(Duration::from_secs(20))
        .connect_timeout(Duration::from_secs(10))
                .gzip(true)
        .build()?;
    let _ = CLIENT.set(c);
    Ok(CLIENT.get().unwrap())
}
