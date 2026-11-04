use crownos_apps_database::AppId;
use crownui::prelude::Cx;

use super::Store;
use crate::catalog::load_icon;

/// Decoded sizes, in device pixels; a view picks the smallest that covers it at 2×.
const EDGES: [u32; 3] = [64, 128, 256];
const DEVICE_SCALE: f32 = 2.0;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IconKey {
    pub app: AppId,
    pub edge: u32,
}

impl IconKey {
    pub fn new(app: AppId, logical_size: f32) -> Self {
        let wanted = (logical_size * DEVICE_SCALE).ceil() as u32;
        let edge = EDGES
            .into_iter()
            .find(|&edge| edge >= wanted)
            .unwrap_or(EDGES[EDGES.len() - 1]);
        Self { app, edge }
    }
}

impl Store {
    /// Starts fetching and decoding `relative` for `key` unless it is loaded or on its way.
    pub fn request_icon(self, cx: &mut Cx, key: IconKey, relative: String) {
        if !self
            .requested_icons
            .update(cx, |requested| requested.insert(key.clone()))
        {
            return;
        }
        let edge = key.edge;
        let decode = async move {
            tokio::task::spawn_blocking(move || load_icon(&relative, edge))
                .await
                .ok()
                .flatten()
        };
        cx.spawn(decode, move |cx, pixels| {
            let Some(pixels) = pixels else {
                return;
            };
            let image = cx.register_image(pixels.width, pixels.height, pixels.rgba);
            self.icons.update(cx, |icons| {
                icons.insert(key, image);
            });
        });
    }
}
