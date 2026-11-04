//! The one [`Store`] every view is handed: signals fed by the catalog sync and
//! the queue worker.

mod catalog_sync;
mod icons;
mod jobs;
mod page;
mod sources;

use std::collections::{HashMap, HashSet};

use crownos_apps_database::AppId;
use crownui::{
    ext::ImageId,
    kit::Toasts,
    prelude::{Cx, Signal, Store as Slot},
};
use tokio::sync::mpsc::UnboundedSender;

pub use catalog_sync::CatalogStatus;
pub use icons::IconKey;
pub use page::Page;
pub use sources::SourceStates;

use crate::{
    catalog::Catalog,
    queue::{JobEntry, QueueCommand, run_queue},
    util::worker::spawn_worker,
};

#[derive(Clone, Copy)]
pub struct Store {
    pub catalog: Signal<Catalog>,
    pub catalog_status: Signal<CatalogStatus>,
    pub query: Signal<String>,
    pub page: Signal<usize>,
    pub previous_page: Signal<usize>,
    pub selected: Signal<Option<AppId>>,
    pub sources: Signal<SourceStates>,
    pub jobs: Signal<Vec<JobEntry>>,
    pub icons: Signal<HashMap<IconKey, ImageId>>,
    requested_icons: Slot<HashSet<IconKey>>,
    commands: Slot<Option<UnboundedSender<QueueCommand>>>,
    next_job: Slot<u64>,
    toasts: Toasts,
}

impl Store {
    pub fn new(cx: &mut Cx, toasts: Toasts) -> Self {
        Self {
            catalog: cx.signal(Catalog::default()),
            catalog_status: cx.signal(CatalogStatus::Loading),
            query: cx.signal(String::new()),
            page: cx.signal(Page::Discover.ordinal()),
            previous_page: cx.signal(Page::Discover.ordinal()),
            selected: cx.signal(None),
            sources: cx.signal(SourceStates::default()),
            jobs: cx.signal(Vec::new()),
            icons: cx.signal(HashMap::new()),
            requested_icons: cx.store(HashSet::new()),
            commands: cx.store(None),
            next_job: cx.store(0),
            toasts,
        }
    }

    /// Loads the catalog and starts the queue worker for as long as the window lives.
    pub fn start(self, cx: &mut Cx) {
        self.sync_catalog(cx);
        let commands = spawn_worker(cx, run_queue, move |cx, event| self.apply(cx, event));
        self.attach_queue(cx, commands);
    }

    /// Routes queued jobs to `commands`, and follows the search from here on.
    pub fn attach_queue(self, cx: &mut Cx, commands: UnboundedSender<QueueCommand>) {
        self.commands.update(cx, |slot| *slot = Some(commands));
        self.follow_search(cx);
    }
}
