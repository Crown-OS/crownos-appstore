use std::{sync::LazyLock, time::Duration};

use thiserror::Error;
use ureq::Agent;

const TIMEOUT: Duration = Duration::from_secs(20);
const MAX_BODY_BYTES: u64 = 64 * 1024 * 1024;
const NOT_MODIFIED: u16 = 304;

static AGENT: LazyLock<Agent> = LazyLock::new(|| {
    Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(TIMEOUT))
        .user_agent(concat!("crownos-appstore/", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
});

pub enum Fetched {
    NotModified,
    Body {
        bytes: Vec<u8>,
        etag: Option<String>,
    },
}

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("{0}")]
    Transport(#[from] ureq::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("server answered {0}")]
    Status(u16),
}

/// GETs `url`, revalidating against `etag`. `file://` URLs read the file, so a
/// locally built catalog can be tested without a server.
pub fn fetch(url: &str, etag: Option<&str>) -> Result<Fetched, FetchError> {
    if let Some(path) = url.strip_prefix("file://") {
        let bytes = std::fs::read(path)?;
        return Ok(Fetched::Body { bytes, etag: None });
    }
    let mut request = AGENT.get(url);
    if let Some(etag) = etag {
        request = request.header("If-None-Match", etag);
    }
    let mut response = request.call()?;
    match response.status().as_u16() {
        NOT_MODIFIED => Ok(Fetched::NotModified),
        _ if response.status().is_success() => {
            let etag = response
                .headers()
                .get("etag")
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned);
            let bytes = response
                .body_mut()
                .with_config()
                .limit(MAX_BODY_BYTES)
                .read_to_vec()?;
            Ok(Fetched::Body { bytes, etag })
        }
        status => Err(FetchError::Status(status)),
    }
}
