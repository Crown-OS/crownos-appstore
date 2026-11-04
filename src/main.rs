//! # CrownOS App Store
//!
//! * [`catalog`] — the app database: download, cache and search.
//! * [`source`] — one backend per package manager behind the `Source` trait.
//! * [`queue`] — the worker that runs installs, removals and updates one at a time.
//! * [`state`] — the `Store` every view is handed.
//! * [`views`], [`app`] — the pages and the window.

mod app;
mod catalog;
mod queue;
mod source;
mod state;
mod util;
mod views;

use anyhow::{Result, bail};
use crownui::{
    AppConfig,
    kit::{INTER_FAMILY, inter_font},
    prelude::BLACK,
};

const INITIAL_SIZE: (u32, u32) = (1180, 780);
const MIN_SIZE: (u32, u32) = (820, 560);

fn main() -> Result<()> {
    if rustix::process::geteuid().is_root() {
        bail!("The App Store runs as your user; it asks for a password when an action needs one.");
    }
    let config = AppConfig::new("App Store")
        .app_id("crownappstore")
        .size(INITIAL_SIZE.0, INITIAL_SIZE.1)
        .min_size(MIN_SIZE.0, MIN_SIZE.1)
        .clear_color(BLACK)
        .mcp(cfg!(debug_assertions))
        .font(inter_font().to_vec(), Some(INTER_FAMILY));
    Ok(crownui::run(app::root(), config)?)
}
