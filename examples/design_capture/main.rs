//! Captures every App Store state for the CrownOS-Design Figma file over the published catalog
//! and a sample package library. Run both appearances with
//! `../crownui/capture/scripts/capture-app.sh .`; output lands in `$CROWN_DESIGN_DIR`.

#![allow(dead_code)]

#[path = "../../src/app/config.rs"]
mod app_config;
#[path = "../../src/catalog/mod.rs"]
mod catalog;
#[path = "../../src/queue/mod.rs"]
mod queue;
mod sample;
#[path = "../../src/app/shell.rs"]
mod shell;
#[path = "../../src/source/mod.rs"]
mod source;
#[path = "../../src/state/mod.rs"]
mod state;
#[path = "../../src/util/mod.rs"]
mod util;
#[path = "../../src/views/mod.rs"]
mod views;

use std::{cell::Cell, rc::Rc, time::Duration};

use capture::{CaptureApp, Carrier};
use crownconfig::load_appearance;
use crownos_apps_database::{AppId, Category, SourceKind};
use crowntest::{Role, role};
use crownui::{
    kit::{
        Toast, Toasts, WindowRadius, provide_configured_theme, provide_window_radius, use_toasts,
    },
    prelude::component,
};

use crate::{
    queue::{JobId, QueueEvent},
    sample::{Library, Sample},
    source::Output,
    state::{CatalogStatus, Page, Store},
    views::source_name,
};

const OFFLINE_ERROR: &str = "download failed: host not found";
const TOAST_LIFETIME: Duration = Duration::from_secs(3600);

struct Window {
    app: CaptureApp,
    store: Store,
    toasts: Toasts,
}

impl Window {
    fn open(sample: &Rc<Sample>, status: CatalogStatus, library: Library) -> anyhow::Result<Self> {
        let mounted = Rc::new(Cell::new(None));
        let root = component({
            let mounted = Rc::clone(&mounted);
            let sample = Rc::clone(sample);
            move |cx| {
                provide_configured_theme(cx);
                provide_window_radius(cx, WindowRadius(f32::from(load_appearance().border_radius)));
                let toasts = use_toasts(cx);
                let store = sample.store(cx, toasts, status.clone(), library);
                mounted.set(Some((store, toasts)));
                shell::shell(cx, store)
            }
        });
        let app = capture::mount_app(root, &app_config::app_config(), None)?;
        let (store, toasts) = mounted
            .get()
            .ok_or_else(|| anyhow::anyhow!("the window did not mount"))?;
        Ok(Self { app, store, toasts })
    }

    fn show(&mut self, page: Page) -> &mut Self {
        self.store.show(self.app.ui().cx(), page);
        self.app.settle();
        self
    }

    fn open_app(&mut self, id: &str) -> anyhow::Result<&mut Self> {
        let id = AppId::try_from(id.to_owned())?;
        self.store.show(self.app.ui().cx(), Page::Discover);
        self.store.open_app(self.app.ui().cx(), id);
        self.app.settle();
        Ok(self)
    }

    fn click(&mut self, query_role: Role, name: &str) -> &mut Self {
        self.app.click(role(query_role).name(name)).settle();
        self
    }

    fn report(&mut self, events: impl IntoIterator<Item = QueueEvent>) -> &mut Self {
        for event in events {
            self.store.apply(self.app.ui().cx(), event);
        }
        self.app.settle();
        self
    }

    fn toast(&mut self, message: &str) -> &mut Self {
        let cx = self.app.ui().cx();
        self.toasts.dismiss_all(cx);
        self.toasts
            .show_with(cx, Toast::new(message).duration(TOAST_LIFETIME));
        self
    }
}

fn line(job: u64, text: &str) -> QueueEvent {
    QueueEvent::Output(JobId(job), Output::Line(text.to_owned()))
}

fn main() -> anyhow::Result<()> {
    let sample = Rc::new(Sample::load()?);
    let mut carrier = Carrier::new(
        "App Store",
        "crownos-appstore: the CrownOS App Store window over the published catalog and a sample package library, captured headlessly from the current source.",
    );
    browse(&sample, &mut carrier)?;
    search(&sample, &mut carrier)?;
    notifications(&sample, &mut carrier)?;
    empty_and_errors(&sample, &mut carrier)?;
    let path = carrier.write(None)?;
    println!("{}", path.display());
    capture::finish()
}

fn browse(sample: &Rc<Sample>, carrier: &mut Carrier) -> anyhow::Result<()> {
    let mut window = Window::open(sample, CatalogStatus::Ready, Library::Used)?;
    carrier.capture_state(
        &mut window.app,
        "Store pages",
        "Discover",
        "Every catalog app as a grid of cards",
    )?;
    window
        .app
        .hover(role(Role::Button).name("Firefox"))
        .settle();
    carrier.capture_state(
        &mut window.app,
        "Store pages",
        "Card hover",
        "A card washed in its app's color under the pointer",
    )?;
    window.app.pointer_move(1170.0, 770.0).settle();

    for category in Category::ALL {
        window.show(Page::Category(category));
        carrier.capture_state(
            &mut window.app,
            "Categories",
            category.label(),
            &format!("The {} category", category.label()),
        )?;
    }

    window.open_app("firefox")?;
    carrier.capture_state(
        &mut window.app,
        "App detail",
        "Installed app",
        "Firefox installed from Pacman, offering removal",
    )?;

    window.open_app("steam")?.click(Role::Button, "Remove");
    window.report([
        QueueEvent::Started(JobId(1)),
        line(1, "removing steam..."),
        QueueEvent::Finished(JobId(1), Ok(())),
    ]);
    window.store.sources.update(window.app.ui().cx(), |states| {
        states.get_mut(SourceKind::Pacman).installed.remove("steam");
    });

    window.open_app("zoom")?.click(Role::Button, "Install");
    window.report([
        QueueEvent::Started(JobId(2)),
        line(2, ":: Synchronizing package databases..."),
        QueueEvent::Finished(JobId(2), Err("Authentication was cancelled".to_owned())),
    ]);

    window.open_app("blender")?;
    carrier.capture_state(
        &mut window.app,
        "App detail",
        "Available",
        "Blender with a source picker and Install",
    )?;
    window.click(Role::RadioButton, source_name(SourceKind::Snap));
    carrier.capture_state(
        &mut window.app,
        "App detail",
        "Needs setup",
        "Snap picked before it is set up: Set Up & Install",
    )?;
    window
        .click(Role::RadioButton, source_name(SourceKind::Pacman))
        .click(Role::Button, "Install");
    carrier.capture_state(
        &mut window.app,
        "App detail",
        "Waiting in queue",
        "The install waits behind other jobs, cancellable",
    )?;
    window.report([
        QueueEvent::Started(JobId(3)),
        line(3, "resolving dependencies..."),
    ]);
    carrier.capture_state(
        &mut window.app,
        "App detail",
        "Preparing",
        "Running before the package manager reports progress",
    )?;
    window.report([
        line(3, "(3/4) installing blender"),
        QueueEvent::Output(JobId(3), Output::Progress(0.46)),
    ]);
    carrier.capture_state(
        &mut window.app,
        "App detail",
        "Installing",
        "A progress bar with the latest output line",
    )?;

    window.open_app("kdenlive")?.click(Role::Button, "Install");
    window
        .open_app("krita")?
        .click(Role::RadioButton, source_name(SourceKind::Snap))
        .click(Role::Button, "Set Up & Install");
    window.store.cancel(window.app.ui().cx(), JobId(4));
    window.show(Page::Queue);
    carrier.capture_state(
        &mut window.app,
        "Store pages",
        "Queue",
        "Done, failed, running, cancelled and waiting jobs",
    )?;

    window.show(Page::Installed);
    carrier.capture_state(
        &mut window.app,
        "Store pages",
        "Installed",
        "Catalog apps found installed by any source",
    )?;

    window.show(Page::Updates);
    carrier.capture_state(
        &mut window.app,
        "Store pages",
        "Updates",
        "Pending updates grouped by source",
    )?;
    window
        .store
        .update_source(window.app.ui().cx(), SourceKind::Flatpak);
    window.app.settle();
    carrier.capture_state(
        &mut window.app,
        "Store pages",
        "Updating a source",
        "Flatpak updates queued, its group spinning",
    )?;
    Ok(())
}

fn search(sample: &Rc<Sample>, carrier: &mut Carrier) -> anyhow::Result<()> {
    let mut window = Window::open(sample, CatalogStatus::Ready, Library::Used)?;
    window
        .app
        .focus(role(Role::SearchField))
        .type_text("browser")
        .settle();
    carrier.capture_state(
        &mut window.app,
        "Search",
        "Search results",
        "Apps matching a typed search, best first",
    )?;
    window
        .store
        .query
        .set(window.app.ui().cx(), "photoshop".to_owned());
    window.app.settle();
    carrier.capture_state(
        &mut window.app,
        "Search",
        "No results",
        "A search nothing in the catalog matches",
    )?;
    Ok(())
}

fn notifications(sample: &Rc<Sample>, carrier: &mut Carrier) -> anyhow::Result<()> {
    let mut window = Window::open(sample, CatalogStatus::Ready, Library::Used)?;
    window.toast("Installed Blender");
    carrier.capture_state(
        &mut window.app,
        "Notifications",
        "Job finished",
        "The toast after a job succeeds",
    )?;
    window.toast("Couldn't install Zoom: Authentication was cancelled");
    carrier.capture_state(
        &mut window.app,
        "Notifications",
        "Job failed",
        "The toast after a job fails",
    )?;
    Ok(())
}

fn empty_and_errors(sample: &Rc<Sample>, carrier: &mut Carrier) -> anyhow::Result<()> {
    {
        let mut window = Window::open(sample, CatalogStatus::Loading, Library::Used)?;
        carrier.capture_state(
            &mut window.app,
            "Empty & error states",
            "Loading catalog",
            "The first catalog download in progress",
        )?;
    }
    {
        let mut window = Window::open(
            sample,
            CatalogStatus::Failed(OFFLINE_ERROR.to_owned()),
            Library::Used,
        )?;
        carrier.capture_state(
            &mut window.app,
            "Empty & error states",
            "Catalog offline",
            "No cached catalog and no mirror reachable",
        )?;
    }
    let mut window = Window::open(sample, CatalogStatus::Ready, Library::Fresh)?;
    window.show(Page::Installed);
    carrier.capture_state(
        &mut window.app,
        "Empty & error states",
        "Nothing installed",
        "No catalog app installed yet",
    )?;
    window.show(Page::Updates);
    carrier.capture_state(
        &mut window.app,
        "Empty & error states",
        "Up to date",
        "No source has pending updates",
    )?;
    window.show(Page::Queue);
    carrier.capture_state(
        &mut window.app,
        "Empty & error states",
        "Empty queue",
        "No installs, removals or updates",
    )?;
    Ok(())
}
