//! Runs one whitelisted package operation as root, started through `pkexec`
//! by the App Store. Every package name is validated before anything runs.

use std::{
    io,
    os::unix::{fs::symlink, process::CommandExt},
    process::{Command, ExitCode},
};

use crownos_apps_database::{CatalogError, PackageName};
use crownos_appstore::helper_protocol::{Arity, HelperVerb, UnknownVerb};

const PACMAN: &str = "/usr/bin/pacman";
const SNAP: &str = "/usr/bin/snap";
const SYSTEMCTL: &str = "/usr/bin/systemctl";
const SNAP_MOUNTS: &str = "/var/lib/snapd/snap";
const SNAP_CLASSIC_LINK: &str = "/snap";

#[derive(Debug, thiserror::Error)]
enum HelperError {
    #[error("usage: crownos-appstore-helper <verb> [package...]")]
    MissingVerb,
    #[error(transparent)]
    UnknownVerb(#[from] UnknownVerb),
    #[error(transparent)]
    InvalidPackage(#[from] CatalogError),
    #[error("{verb} takes {expected} package names, got {got}")]
    WrongArity {
        verb: &'static str,
        expected: &'static str,
        got: usize,
    },
    #[error("{0}")]
    Io(#[from] io::Error),
}

fn main() -> ExitCode {
    let Err(error) = run();
    eprintln!("crownos-appstore-helper: {error}");
    ExitCode::from(2)
}

fn run() -> Result<std::convert::Infallible, HelperError> {
    let mut arguments = std::env::args().skip(1);
    let verb: HelperVerb = arguments.next().ok_or(HelperError::MissingVerb)?.parse()?;
    let packages = arguments
        .map(PackageName::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    check_arity(verb, packages.len())?;
    if verb == HelperVerb::EnableSnapd {
        link_classic_snaps()?;
    }
    Err(command(verb, &packages).exec().into())
}

fn check_arity(verb: HelperVerb, got: usize) -> Result<(), HelperError> {
    let (fits, expected) = match verb.arity() {
        Arity::None => (got == 0, "no"),
        Arity::One => (got == 1, "exactly one"),
        Arity::Many => (got > 0, "one or more"),
    };
    if fits {
        Ok(())
    } else {
        Err(HelperError::WrongArity {
            verb: verb.name(),
            expected,
            got,
        })
    }
}

fn command(verb: HelperVerb, packages: &[PackageName]) -> Command {
    let (program, flags): (&str, &[&str]) = match verb {
        HelperVerb::Install => (PACMAN, &["-S", "--needed", "--noconfirm", "--"]),
        HelperVerb::Remove => (PACMAN, &["-Rns", "--noconfirm", "--"]),
        HelperVerb::Upgrade => (PACMAN, &["-Syu", "--noconfirm"]),
        HelperVerb::SnapInstall => (SNAP, &["install"]),
        HelperVerb::SnapInstallClassic => (SNAP, &["install", "--classic"]),
        HelperVerb::SnapRemove => (SNAP, &["remove"]),
        HelperVerb::SnapRefresh => (SNAP, &["refresh"]),
        HelperVerb::EnableSnapd => (SYSTEMCTL, &["enable", "--now", "snapd.socket"]),
    };
    let mut command = Command::new(program);
    command
        .args(flags)
        .args(packages.iter().map(PackageName::as_str));
    command
}

/// Classic snaps expect their mounts under `/snap`.
fn link_classic_snaps() -> io::Result<()> {
    match symlink(SNAP_MOUNTS, SNAP_CLASSIC_LINK) {
        Err(error) if error.kind() != io::ErrorKind::AlreadyExists => Err(error),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arguments(command: &Command) -> Vec<String> {
        command
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn package_operations_end_options_before_names() -> Result<(), HelperError> {
        let packages = [PackageName::try_from("firefox")?];
        let install = command(HelperVerb::Install, &packages);
        assert_eq!(install.get_program(), PACMAN);
        assert_eq!(
            arguments(&install),
            ["-S", "--needed", "--noconfirm", "--", "firefox"]
        );
        let classic = command(HelperVerb::SnapInstallClassic, &packages);
        assert_eq!(arguments(&classic), ["install", "--classic", "firefox"]);
        Ok(())
    }

    #[test]
    fn arity_is_enforced() {
        assert!(check_arity(HelperVerb::Upgrade, 0).is_ok());
        assert!(check_arity(HelperVerb::Upgrade, 1).is_err());
        assert!(check_arity(HelperVerb::SnapInstall, 2).is_err());
        assert!(check_arity(HelperVerb::Install, 0).is_err());
    }

    #[test]
    fn hostile_names_never_reach_a_command() {
        assert!(PackageName::try_from("--overwrite=*").is_err());
        assert!(PackageName::try_from("a;reboot").is_err());
    }
}
