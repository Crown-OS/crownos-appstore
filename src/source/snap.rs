use std::{collections::HashSet, path::Path};

use crownos_apps_database::{PackageName, Source as CatalogSource, SourceKind};
use crownos_appstore::helper_protocol::HelperVerb;

use super::{Invocation, PendingUpdate, Source, SourceError, lines, process::capture};
use crate::util::executable::is_on_path;

const PROGRAM: &str = "snap";
const PACKAGE: &str = "snapd";
const SOCKET: &str = "/run/snapd.socket";
const UP_TO_DATE: &str = "All snaps up to date";

/// The Snap Store, through snapd.
pub struct Snap;

/// Rows of a `snap` table, without its header.
fn rows(output: &str) -> impl Iterator<Item = &str> {
    lines(output).skip(1)
}

impl Source for Snap {
    fn kind(&self) -> SourceKind {
        SourceKind::Snap
    }

    fn setup(&self) -> Vec<Invocation> {
        let mut steps = Vec::new();
        if !is_on_path(PROGRAM) {
            steps.push(Invocation::privileged(HelperVerb::Install).arg(PACKAGE));
        }
        if !steps.is_empty() || !Path::new(SOCKET).exists() {
            steps.push(Invocation::privileged(HelperVerb::EnableSnapd));
        }
        steps
    }

    fn install(&self, source: &CatalogSource) -> Invocation {
        let verb = if source.classic {
            HelperVerb::SnapInstallClassic
        } else {
            HelperVerb::SnapInstall
        };
        Invocation::privileged(verb).arg(source.package.to_string())
    }

    fn remove(&self, package: &PackageName) -> Invocation {
        Invocation::privileged(HelperVerb::SnapRemove).arg(package.to_string())
    }

    fn update_all(&self) -> Invocation {
        Invocation::privileged(HelperVerb::SnapRefresh)
    }

    fn installed(&self) -> Result<HashSet<String>, SourceError> {
        let output = capture(PROGRAM, &["list"])?;
        Ok(rows(&output)
            .filter_map(|row| row.split_whitespace().next())
            .map(str::to_owned)
            .collect())
    }

    fn updates(&self) -> Result<Vec<PendingUpdate>, SourceError> {
        let output = capture(PROGRAM, &["refresh", "--list"])?;
        if output.contains(UP_TO_DATE) {
            return Ok(Vec::new());
        }
        Ok(rows(&output)
            .filter_map(|row| {
                let mut columns = row.split_whitespace();
                Some(PendingUpdate {
                    kind: SourceKind::Snap,
                    package: columns.next()?.to_owned(),
                    detail: columns.next().unwrap_or_default().to_owned(),
                })
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classic_snaps_use_the_classic_verb() -> Result<(), crownos_apps_database::CatalogError> {
        let source = CatalogSource {
            kind: SourceKind::Snap,
            package: PackageName::try_from("code")?,
            classic: true,
        };
        let invocation = Snap.install(&source);
        assert_eq!(invocation.program, "pkexec");
        assert_eq!(invocation.arguments[1], "snap-install-classic");
        assert_eq!(invocation.arguments[2], "code");
        Ok(())
    }
}
