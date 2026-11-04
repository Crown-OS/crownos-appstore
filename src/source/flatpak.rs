use std::collections::HashSet;

use crownos_apps_database::{PackageName, Source as CatalogSource, SourceKind};
use crownos_appstore::helper_protocol::HelperVerb;

use super::{
    Invocation, PendingUpdate, Source, SourceError, first_column, lines, process::capture,
};
use crate::util::executable::is_on_path;

const PROGRAM: &str = "flatpak";
const REMOTE: &str = "flathub";
const REMOTE_URL: &str = "https://dl.flathub.org/repo/flathub.flatpakrepo";
const UNATTENDED: [&str; 2] = ["-y", "--noninteractive"];

/// Flathub, installed per user so no password is needed.
pub struct Flatpak;

fn has_user_remote() -> bool {
    capture(PROGRAM, &["remotes", "--user", "--columns=name"])
        .is_ok_and(|output| lines(&output).any(|name| name == REMOTE))
}

fn add_user_remote() -> Invocation {
    Invocation::new(PROGRAM).args([
        "remote-add",
        "--user",
        "--if-not-exists",
        REMOTE,
        REMOTE_URL,
    ])
}

impl Source for Flatpak {
    fn kind(&self) -> SourceKind {
        SourceKind::Flatpak
    }

    fn setup(&self) -> Vec<Invocation> {
        if !is_on_path(PROGRAM) {
            vec![
                Invocation::privileged(HelperVerb::Install).arg(PROGRAM),
                add_user_remote(),
            ]
        } else if has_user_remote() {
            Vec::new()
        } else {
            vec![add_user_remote()]
        }
    }

    fn install(&self, source: &CatalogSource) -> Invocation {
        Invocation::new(PROGRAM)
            .args(["install", "--user"])
            .args(UNATTENDED)
            .arg(REMOTE)
            .arg(source.package.to_string())
    }

    fn remove(&self, package: &PackageName) -> Invocation {
        Invocation::new(PROGRAM)
            .arg("uninstall")
            .args(UNATTENDED)
            .arg(package.to_string())
    }

    fn update_all(&self) -> Invocation {
        Invocation::new(PROGRAM).arg("update").args(UNATTENDED)
    }

    fn installed(&self) -> Result<HashSet<String>, SourceError> {
        capture(PROGRAM, &["list", "--app", "--columns=application"])
            .map(|output| first_column(&output))
    }

    fn updates(&self) -> Result<Vec<PendingUpdate>, SourceError> {
        let output = capture(
            PROGRAM,
            &[
                "remote-ls",
                "--updates",
                "--app",
                "--columns=application,version",
            ],
        )?;
        Ok(lines(&output)
            .filter_map(|line| {
                let mut columns = line.split('\t');
                Some(PendingUpdate {
                    kind: SourceKind::Flatpak,
                    package: columns.next()?.to_owned(),
                    detail: columns.next().unwrap_or_default().to_owned(),
                })
            })
            .collect())
    }
}
