//! One module per package manager behind [`Source`]. Adding a source means a
//! new module, a `SourceKind` variant in the catalog and one arm in [`backend`].

mod aur;
mod error;
mod flatpak;
mod invocation;
mod pacman;
mod pending_update;
mod probe;
mod process;
mod progress;
mod snap;

use std::collections::HashSet;

use crownos_apps_database::{PackageName, Source as CatalogSource, SourceKind};

pub use error::SourceError;
pub use invocation::Invocation;
pub use pending_update::PendingUpdate;
pub use probe::{Probe, probe};
pub use process::{Output, stream};

pub trait Source: Sync {
    fn kind(&self) -> SourceKind;

    /// What has to run before this source can install anything; empty once it is ready.
    fn setup(&self) -> Vec<Invocation>;

    fn install(&self, source: &CatalogSource) -> Invocation;

    fn remove(&self, package: &PackageName) -> Invocation;

    fn update_all(&self) -> Invocation;

    fn installed(&self) -> Result<HashSet<String>, SourceError>;

    fn updates(&self) -> Result<Vec<PendingUpdate>, SourceError>;
}

pub fn backend(kind: SourceKind) -> &'static dyn Source {
    match kind {
        SourceKind::Pacman => &pacman::Pacman,
        SourceKind::Aur => &aur::Aur,
        SourceKind::Flatpak => &flatpak::Flatpak,
        SourceKind::Snap => &snap::Snap,
    }
}

/// Non-empty lines of `output`, trimmed.
fn lines(output: &str) -> impl Iterator<Item = &str> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
}

/// The first column of every line, as `pacman -Qq` and friends print it.
fn first_column(output: &str) -> HashSet<String> {
    lines(output)
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_owned)
        .collect()
}

/// `name old -> new` lines, as `checkupdates` and `paru -Qua` print them.
fn version_changes(kind: SourceKind, output: &str) -> Vec<PendingUpdate> {
    lines(output)
        .filter_map(|line| {
            let (package, change) = line.split_once(' ')?;
            Some(PendingUpdate {
                kind,
                package: package.to_owned(),
                detail: change.replace("->", "→"),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_version_change_lines() {
        let updates = version_changes(SourceKind::Pacman, "firefox 1.0-1 -> 1.1-1\n\nvlc 3 -> 4\n");
        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].package, "firefox");
        assert_eq!(updates[0].detail, "1.0-1 → 1.1-1");
    }

    #[test]
    fn every_kind_has_its_backend() {
        for kind in SourceKind::ALL {
            assert_eq!(backend(kind).kind(), kind);
        }
    }
}
