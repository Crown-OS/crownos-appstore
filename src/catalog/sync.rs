use crownos_apps_database::{CatalogError, INDEX_FILE, Index};
use thiserror::Error;

use super::mirrors;
use crate::util::{
    cache,
    http::{FetchError, Fetched, fetch},
};

const ETAG_FILE: &str = "index.etag";

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("download failed: {0}")]
    Fetch(#[from] FetchError),
    #[error(transparent)]
    Catalog(#[from] CatalogError),
}

/// The index downloaded last time, if it is still readable.
pub fn load_cached() -> Option<Index> {
    Index::decode(&cache::read(INDEX_FILE)?).ok()
}

/// Downloads the index from the first mirror that answers. `None` means the
/// cached copy is current.
pub fn refresh() -> Result<Option<Index>, SyncError> {
    let etag = cache::read(ETAG_FILE).and_then(|bytes| String::from_utf8(bytes).ok());
    let etag = etag.filter(|_| load_cached().is_some());
    let mut last_error = None;
    for url in mirrors::urls(INDEX_FILE) {
        match download(&url, etag.as_deref()) {
            Ok(index) => return Ok(index),
            Err(error) => last_error = Some(error),
        }
    }
    last_error.map_or(Ok(None), Err)
}

fn download(url: &str, etag: Option<&str>) -> Result<Option<Index>, SyncError> {
    let Fetched::Body { bytes, etag } = fetch(url, etag)? else {
        return Ok(None);
    };
    let index = Index::decode(&bytes)?;
    if cache::write(INDEX_FILE, &bytes).is_ok() {
        let _ = cache::write(ETAG_FILE, etag.unwrap_or_default().as_bytes());
    }
    Ok(Some(index))
}
