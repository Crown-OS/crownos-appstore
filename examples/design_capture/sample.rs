use std::{collections::HashSet, path::Path};

use anyhow::{Context, Result};
use crownos_apps_database::{INDEX_FILE, Index, SourceKind};
use crownui::{kit::Toasts, prelude::Cx};
use tokio::sync::mpsc::unbounded_channel;

use crate::{
    catalog::Catalog,
    source::PendingUpdate,
    state::{CatalogStatus, IconKey, SourceStates, Store},
    util::pixels::{Pixels, decode_png},
};

const PUBLISHED_CATALOG: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/database/dist/v1");
const ICON_EDGES: [u32; 2] = [128, 256];

const PACMAN_INSTALLED: [&str; 8] = [
    "firefox",
    "vlc",
    "gimp",
    "obs-studio",
    "thunderbird",
    "steam",
    "linux",
    "mesa",
];
const AUR_INSTALLED: [&str; 2] = ["visual-studio-code-bin", "spotify"];
const FLATPAK_INSTALLED: [&str; 3] = [
    "org.signal.Signal",
    "com.discordapp.Discord",
    "md.obsidian.Obsidian",
];
const PENDING_UPDATES: [(SourceKind, &str, &str); 7] = [
    (SourceKind::Pacman, "firefox", "133.0-1 → 133.0.3-1"),
    (
        SourceKind::Pacman,
        "linux",
        "6.12.1.arch1-1 → 6.12.4.arch1-1",
    ),
    (SourceKind::Pacman, "mesa", "1:24.2.7-1 → 1:24.3.1-1"),
    (SourceKind::Pacman, "vlc", "3.0.21-4 → 3.0.21-5"),
    (
        SourceKind::Aur,
        "visual-studio-code-bin",
        "1.95.3-1 → 1.96.0-1",
    ),
    (SourceKind::Flatpak, "com.discordapp.Discord", "0.0.77"),
    (SourceKind::Flatpak, "org.signal.Signal", "7.36.0"),
];

/// The published catalog with every icon decoded at the sizes the views ask for.
pub struct Sample {
    index: Index,
    icons: Vec<(IconKey, Pixels)>,
}

/// What the package managers report.
#[derive(Clone, Copy)]
pub enum Library {
    Used,
    Fresh,
}

impl Sample {
    pub fn load() -> Result<Self> {
        let published = Path::new(PUBLISHED_CATALOG);
        let bytes = std::fs::read(published.join(INDEX_FILE))
            .context("the catalog is not built; run `cargo build-index`")?;
        let index = Index::decode(&bytes)?;
        let icons = index
            .apps
            .iter()
            .filter_map(|app| {
                let bytes = std::fs::read(published.join(app.icon.as_ref()?)).ok()?;
                Some((app.id.clone(), bytes))
            })
            .flat_map(|(id, bytes)| {
                ICON_EDGES.into_iter().filter_map(move |edge| {
                    let key = IconKey {
                        app: id.clone(),
                        edge,
                    };
                    Some((key, decode_png(&bytes, edge)?))
                })
            })
            .collect();
        Ok(Self { index, icons })
    }

    /// A store over the catalog in `status`, with `library` installed and jobs going nowhere.
    pub fn store(
        &self,
        cx: &mut Cx,
        toasts: Toasts,
        status: CatalogStatus,
        library: Library,
    ) -> Store {
        let store = Store::new(cx, toasts);
        if status == CatalogStatus::Ready {
            store.catalog.set(cx, Catalog::new(self.index.clone()));
            self.register_icons(cx, store);
        }
        store.catalog_status.set(cx, status);
        store.sources.set(cx, sources(library));
        let (commands, _) = unbounded_channel();
        store.attach_queue(cx, commands);
        store
    }

    fn register_icons(&self, cx: &mut Cx, store: Store) {
        let images: Vec<_> = self
            .icons
            .iter()
            .map(|(key, pixels)| {
                let image = cx.register_image(pixels.width, pixels.height, pixels.rgba.clone());
                (key.clone(), image)
            })
            .collect();
        store.icons.update(cx, |icons| icons.extend(images));
    }
}

fn sources(library: Library) -> SourceStates {
    let mut states = SourceStates::default();
    for kind in [SourceKind::Pacman, SourceKind::Aur, SourceKind::Flatpak] {
        states.get_mut(kind).ready = Some(true);
    }
    states.get_mut(SourceKind::Snap).ready = Some(false);
    if let Library::Used = library {
        let installed = [
            (SourceKind::Pacman, &PACMAN_INSTALLED[..]),
            (SourceKind::Aur, &AUR_INSTALLED[..]),
            (SourceKind::Flatpak, &FLATPAK_INSTALLED[..]),
        ];
        for (kind, packages) in installed {
            states.get_mut(kind).installed = packages
                .iter()
                .map(|&package| package.into())
                .collect::<HashSet<_>>();
        }
        for (kind, package, detail) in PENDING_UPDATES {
            states.get_mut(kind).updates.push(PendingUpdate {
                kind,
                package: package.into(),
                detail: detail.into(),
            });
        }
    }
    states
}
