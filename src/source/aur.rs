use std::collections::HashSet;

use crownos_apps_database::{PackageName, Source as CatalogSource, SourceKind};
use crownos_appstore::helper_protocol::HelperVerb;

use super::{
    Invocation, PendingUpdate, Source, SourceError, first_column, process::capture, version_changes,
};
use crate::util::executable::is_on_path;

/// AUR helpers exit 1 when `-Qua` finds nothing.
const NO_UPDATES: i32 = 1;

/// The helper CrownOS ships in `[crownos]`.
const PREFERRED_HELPER: &str = "paru";

/// The AUR, through paru or yay. Both build as the user and elevate with
/// pkexec, so they never wait on a terminal password prompt.
pub struct Aur;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AurHelper {
    Paru,
    Yay,
}

impl AurHelper {
    fn detect() -> Option<Self> {
        [Self::Paru, Self::Yay]
            .into_iter()
            .find(|helper| is_on_path(helper.program()))
    }

    const fn program(self) -> &'static str {
        match self {
            Self::Paru => "paru",
            Self::Yay => "yay",
        }
    }

    const fn unattended_flags(self) -> &'static [&'static str] {
        match self {
            Self::Paru => &["--noconfirm", "--skipreview", "--sudo", "pkexec"],
            Self::Yay => &[
                "--noconfirm",
                "--answerdiff",
                "None",
                "--answerclean",
                "None",
                "--answeredit",
                "None",
                "--sudo",
                "pkexec",
            ],
        }
    }

    fn invocation(self, operation: &'static str) -> Invocation {
        Invocation::new(self.program())
            .arg(operation)
            .args(self.unattended_flags().iter().copied())
    }
}

fn helper() -> AurHelper {
    AurHelper::detect().unwrap_or(AurHelper::Paru)
}

impl Source for Aur {
    fn kind(&self) -> SourceKind {
        SourceKind::Aur
    }

    fn setup(&self) -> Vec<Invocation> {
        match AurHelper::detect() {
            Some(_) => Vec::new(),
            None => vec![Invocation::privileged(HelperVerb::Install).arg(PREFERRED_HELPER)],
        }
    }

    fn install(&self, source: &CatalogSource) -> Invocation {
        helper()
            .invocation("-S")
            .arg("--needed")
            .arg(source.package.to_string())
    }

    fn remove(&self, package: &PackageName) -> Invocation {
        Invocation::privileged(HelperVerb::Remove).arg(package.to_string())
    }

    fn update_all(&self) -> Invocation {
        helper().invocation("-Sua")
    }

    fn installed(&self) -> Result<HashSet<String>, SourceError> {
        capture("pacman", &["-Qmq"]).map(|output| first_column(&output))
    }

    fn updates(&self) -> Result<Vec<PendingUpdate>, SourceError> {
        let Some(helper) = AurHelper::detect() else {
            return Ok(Vec::new());
        };
        match capture(helper.program(), &["-Qua"]) {
            Ok(output) => Ok(version_changes(SourceKind::Aur, &output)),
            Err(error) if error.exit_code() == Some(NO_UPDATES) => Ok(Vec::new()),
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yay_answers_every_prompt_up_front() {
        let invocation = AurHelper::Yay.invocation("-S");
        let line = invocation.to_string();
        assert!(line.starts_with("yay -S --noconfirm"));
        assert!(line.contains("--answerdiff None") && line.ends_with("--sudo pkexec"));
    }
}
