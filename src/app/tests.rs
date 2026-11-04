//! The whole window driven headlessly over a three-app catalog; queued jobs
//! are caught on a channel instead of running.

use crownos_apps_database::SourceKind;
use crowntest::{Role, TestApp, role};
use crownui::{
    kit::{Theme, provide_theme},
    prelude::component,
};
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

use super::{fixtures::fixture_store, shell::shell};
use crate::queue::{Action, Job, QueueCommand};

const WIDTH: f32 = 1180.0;
const HEIGHT: f32 = 780.0;

struct Window {
    app: TestApp,
    queue: UnboundedReceiver<QueueCommand>,
}

impl Window {
    fn open() -> Self {
        let (commands, queue) = unbounded_channel();
        let mut app = TestApp::with_size(
            component(move |cx| {
                provide_theme(cx, Theme::dark());
                let store = fixture_store(cx, commands);
                shell(cx, store)
            }),
            WIDTH,
            HEIGHT,
        );
        app.settle();
        Self { app, queue }
    }

    fn has_card(&mut self, name: &str) -> bool {
        self.app.exists(role(Role::Button).name(name))
    }

    fn click(&mut self, query_role: Role, name: &str) {
        self.app.click(role(query_role).name(name)).settle();
    }

    fn queued(&mut self) -> Vec<Job> {
        std::iter::from_fn(|| self.queue.try_recv().ok())
            .filter_map(|command| match command {
                QueueCommand::Run(job) => Some(job),
                _ => None,
            })
            .collect()
    }
}

#[test]
fn discover_shows_every_app_and_counts_installed_ones() {
    let mut window = Window::open();
    for name in ["Blender", "Firefox", "VLC"] {
        assert!(window.has_card(name), "{name} missing");
    }
    assert!(window.app.exists(role(Role::Text).name("3 apps")));
    assert!(window.app.exists(role(Role::MenuItem).name("Installed 1")));
    assert!(window.app.exists(role(Role::MenuItem).name("Updates 1")));
}

#[test]
fn categories_filter_the_grid() {
    let mut window = Window::open();
    window.click(Role::MenuItem, "Graphics");
    assert!(window.has_card("Blender"));
    assert!(!window.has_card("Firefox"));
}

#[test]
fn searching_narrows_the_grid() {
    let mut window = Window::open();
    window
        .app
        .focus(role(Role::SearchField))
        .type_text("vl")
        .settle();
    assert!(window.has_card("VLC"));
    assert!(!window.has_card("Firefox"));
}

#[test]
fn installing_from_an_unready_source_sets_it_up_first() {
    let mut window = Window::open();
    window.click(Role::Button, "Blender");
    window.click(Role::Button, "Set Up & Install");
    let jobs = window.queued();
    assert_eq!(jobs.len(), 2);
    assert_eq!(
        (jobs[0].kind, &jobs[0].action),
        (SourceKind::Snap, &Action::Setup)
    );
    assert!(
        matches!(&jobs[1].action, Action::Install(source) if source.package.as_str() == "blender")
    );
    assert!(window.app.exists(role(Role::MenuItem).name("Queue 2")));
}

#[test]
fn an_installed_app_offers_removal_from_its_source() {
    let mut window = Window::open();
    window.click(Role::Button, "Firefox");
    window.click(Role::Button, "Remove");
    let jobs = window.queued();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].kind, SourceKind::Pacman);
    assert!(matches!(&jobs[0].action, Action::Remove(package) if package.as_str() == "firefox"));
}

#[test]
fn the_installed_page_lists_installed_apps_only() {
    let mut window = Window::open();
    window.click(Role::MenuItem, "Installed 1");
    assert!(window.app.exists(role(Role::ListItem).name("Firefox")));
    assert!(!window.app.exists(role(Role::ListItem).name("VLC")));
    window.click(Role::Button, "Remove");
    assert_eq!(window.queued().len(), 1);
}

#[test]
fn back_returns_to_the_page_the_app_was_opened_from() {
    let mut window = Window::open();
    window.click(Role::MenuItem, "Internet");
    window.click(Role::Button, "Firefox");
    window.click(Role::Button, "Back");
    assert!(window.app.exists(role(Role::Text).name("Internet")));
    assert!(window.has_card("Firefox"));
}

#[test]
fn update_all_queues_every_source_with_updates() {
    let mut window = Window::open();
    window.click(Role::MenuItem, "Updates 1");
    assert!(
        window
            .app
            .exists(role(Role::Text).name("1 update available"))
    );
    window.click(Role::Button, "Update All");
    let jobs = window.queued();
    assert_eq!(jobs.len(), 1);
    assert_eq!(
        (jobs[0].kind, &jobs[0].action),
        (SourceKind::Pacman, &Action::UpdateAll)
    );
}

#[test]
fn waiting_jobs_can_be_cancelled_from_the_queue() {
    let mut window = Window::open();
    window.click(Role::Button, "VLC");
    window.click(Role::Button, "Install");
    window.click(Role::MenuItem, "Queue 1");
    assert!(window.app.exists(role(Role::Text).name("Install VLC")));
    window.click(Role::Button, "Cancel");
    assert!(window.app.exists(role(Role::Text).name("Cancelled")));
    assert!(!window.app.exists(role(Role::MenuItem).name("Queue 1")));
}
