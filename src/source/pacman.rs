use std::collections::HashSet;

use crownos_apps_database::{PackageName, Source as CatalogSource, SourceKind};
use crownos_appstore::helper_protocol::HelperVerb;

use super::{
    Invocation, PendingUpdate, Source, SourceError, first_column, process::capture, version_changes,
};

/// `checkupdates` exits 2 when there is nothing to update.
const NO_UPDATES: i32 = 2;

/// The configured pacman repositories: Arch, CachyOS and `[crownos]`.
pub struct Pacman;

impl Source for Pacman {
    fn kind(&self) -> SourceKind {
        SourceKind::Pacman
    }

    fn setup(&self) -> Vec<Invocation> {
        Vec::new()
    }

    /// Syncs first, so a repository never synced on this system (`multilib`
    /// on a fresh install) is found, and upgrades with it: Arch supports no
    /// partial upgrades.
    fn install(&self, source: &CatalogSource) -> Invocation {
        Invocation::privileged(HelperVerb::Install).arg(source.package.to_string())
    }

    fn remove(&self, package: &PackageName) -> Invocation {
        Invocation::privileged(HelperVerb::Remove).arg(package.to_string())
    }

    /// Arch supports no partial upgrades, so updating anything updates everything.
    fn update_all(&self) -> Invocation {
        Invocation::privileged(HelperVerb::Upgrade)
    }

    fn installed(&self) -> Result<HashSet<String>, SourceError> {
        capture("pacman", &["-Qnq"]).map(|output| first_column(&output))
    }

    fn updates(&self) -> Result<Vec<PendingUpdate>, SourceError> {
        match capture("checkupdates", &[]) {
            Ok(output) => Ok(version_changes(SourceKind::Pacman, &output)),
            Err(error) if error.exit_code() == Some(NO_UPDATES) => Ok(Vec::new()),
            Err(error) => Err(error),
        }
    }
}
