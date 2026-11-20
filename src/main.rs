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
use state::Page;

const PAGE_FLAG: &str = "--page";
const PAGE_FLAG_WITH_VALUE: &str = "--page=";

/// `--page updates` opens the Updates page; anything else is ignored, since a
/// launcher may forward arguments this app has never heard of.
fn initial_page(mut arguments: impl Iterator<Item = String>) -> Page {
    let mut requested = None;
    while let Some(argument) = arguments.next() {
        if argument == PAGE_FLAG {
            requested = arguments.next();
        } else if let Some(value) = argument.strip_prefix(PAGE_FLAG_WITH_VALUE) {
            requested = Some(value.to_owned());
        }
    }
    match requested.as_deref() {
        Some("updates") => Page::Updates,
        _ => Page::Discover,
    }
}

fn main() -> Result<()> {
    if rustix::process::geteuid().is_root() {
        bail!("The App Store runs as your user; it asks for a password when an action needs one.");
    }
    Ok(crownui::run(
        app::root(initial_page(std::env::args().skip(1))),
        app::app_config(),
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(arguments: &[&str]) -> Page {
        initial_page(arguments.iter().map(|argument| (*argument).to_owned()))
    }

    #[test]
    fn opens_the_page_asked_for() {
        assert_eq!(page(&["--page", "updates"]), Page::Updates);
        assert_eq!(page(&["--page=updates"]), Page::Updates);
    }

    #[test]
    fn anything_else_opens_discover() {
        assert_eq!(page(&[]), Page::Discover);
        assert_eq!(page(&["--page", "nonsense"]), Page::Discover);
        assert_eq!(page(&["--page"]), Page::Discover);
        assert_eq!(page(&["stray", "--unknown"]), Page::Discover);
    }
}
