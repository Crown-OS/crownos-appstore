//! A three-app catalog and a store over it, for driving the window headlessly.

use crownos_apps_database::{App, Category, Index, PackageName, Source, SourceKind, ThemeColor};
use crownui::{kit::use_toasts, prelude::Cx};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    catalog::Catalog,
    queue::QueueCommand,
    source::PendingUpdate,
    state::{CatalogStatus, Store},
};

fn source(kind: SourceKind, package: &str) -> Source {
    Source {
        kind,
        package: PackageName::try_from(package).unwrap_or_else(|_| unreachable!()),
        classic: false,
    }
}

fn app(id: &str, name: &str, category: Category, sources: Vec<Source>) -> App {
    App {
        id: id.to_owned().try_into().unwrap_or_else(|_| unreachable!()),
        name: name.into(),
        summary: format!("{name} summary"),
        description: format!("{name} description"),
        category,
        theme_color: ThemeColor(0xff7139),
        icon: None,
        homepage: None,
        license: Some("MIT".into()),
        keywords: Vec::new(),
        sources,
    }
}

pub fn fixture_index() -> Index {
    Index::new(vec![
        app(
            "blender",
            "Blender",
            Category::Graphics,
            vec![source(SourceKind::Snap, "blender")],
        ),
        app(
            "firefox",
            "Firefox",
            Category::Internet,
            vec![source(SourceKind::Pacman, "firefox")],
        ),
        app(
            "vlc",
            "VLC",
            Category::Multimedia,
            vec![
                source(SourceKind::Flatpak, "org.videolan.VLC"),
                source(SourceKind::Pacman, "vlc"),
            ],
        ),
    ])
}

/// Pacman and Flatpak ready, Snap not set up, Firefox installed with an update pending.
pub fn fixture_store(cx: &mut Cx, commands: UnboundedSender<QueueCommand>) -> Store {
    let toasts = use_toasts(cx);
    let store = Store::new(cx, toasts);
    store.catalog.set(cx, Catalog::new(fixture_index()));
    store.catalog_status.set(cx, CatalogStatus::Ready);
    store.sources.update(cx, |states| {
        states.get_mut(SourceKind::Pacman).ready = Some(true);
        states
            .get_mut(SourceKind::Pacman)
            .installed
            .insert("firefox".into());
        states
            .get_mut(SourceKind::Pacman)
            .updates
            .push(PendingUpdate {
                kind: SourceKind::Pacman,
                package: "firefox".into(),
                detail: "1.0 → 1.1".into(),
            });
        states.get_mut(SourceKind::Flatpak).ready = Some(true);
        states.get_mut(SourceKind::Snap).ready = Some(false);
    });
    store.attach_queue(cx, commands);
    store
}
