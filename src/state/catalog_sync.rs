use crownui::prelude::Cx;

use super::Store;
use crate::catalog::{Catalog, load_cached, refresh};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatalogStatus {
    Loading,
    Ready,
    Failed(String),
}

impl Store {
    /// Shows the cached catalog at once, then replaces it with a fresh download.
    pub fn sync_catalog(self, cx: &mut Cx) {
        self.catalog_status.set(cx, CatalogStatus::Loading);
        let cached = async {
            tokio::task::spawn_blocking(load_cached)
                .await
                .ok()
                .flatten()
        };
        cx.spawn(cached, move |cx, cached| {
            let has_cache = cached.is_some();
            if let Some(index) = cached {
                self.catalog.set(cx, Catalog::new(index));
                self.catalog_status.set(cx, CatalogStatus::Ready);
            }
            let fresh = async { tokio::task::spawn_blocking(refresh).await };
            cx.spawn(fresh, move |cx, fresh| match fresh {
                Ok(Ok(Some(index))) => {
                    self.catalog.set(cx, Catalog::new(index));
                    self.catalog_status.set(cx, CatalogStatus::Ready);
                }
                Ok(Ok(None)) => self.catalog_status.set(cx, CatalogStatus::Ready),
                Ok(Err(_)) | Err(_) if has_cache => {}
                Ok(Err(error)) => self
                    .catalog_status
                    .set(cx, CatalogStatus::Failed(error.to_string())),
                Err(error) => self
                    .catalog_status
                    .set(cx, CatalogStatus::Failed(error.to_string())),
            });
        });
    }
}
