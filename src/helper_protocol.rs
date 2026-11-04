//! The fixed set of root operations `crownos-appstore-helper` performs.

use std::str::FromStr;

/// Where packaging installs the helper (every CrownOS binary lands in /usr/bin); `CROWNOS_APPSTORE_HELPER` overrides it in development.
pub const HELPER_PATH: &str = "/usr/bin/crownos-appstore-helper";
pub const HELPER_OVERRIDE: &str = "CROWNOS_APPSTORE_HELPER";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HelperVerb {
    Install,
    Remove,
    Upgrade,
    SnapInstall,
    SnapInstallClassic,
    SnapRemove,
    SnapRefresh,
    EnableSnapd,
}

/// How many package names a verb takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arity {
    None,
    One,
    Many,
}

impl HelperVerb {
    pub const ALL: [Self; 8] = [
        Self::Install,
        Self::Remove,
        Self::Upgrade,
        Self::SnapInstall,
        Self::SnapInstallClassic,
        Self::SnapRemove,
        Self::SnapRefresh,
        Self::EnableSnapd,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Install => "install",
            Self::Remove => "remove",
            Self::Upgrade => "upgrade",
            Self::SnapInstall => "snap-install",
            Self::SnapInstallClassic => "snap-install-classic",
            Self::SnapRemove => "snap-remove",
            Self::SnapRefresh => "snap-refresh",
            Self::EnableSnapd => "enable-snapd",
        }
    }

    pub const fn arity(self) -> Arity {
        match self {
            Self::Install | Self::Remove => Arity::Many,
            Self::SnapInstall | Self::SnapInstallClassic | Self::SnapRemove => Arity::One,
            Self::Upgrade | Self::SnapRefresh | Self::EnableSnapd => Arity::None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unknown helper verb {0:?}")]
pub struct UnknownVerb(String);

impl FromStr for HelperVerb {
    type Err = UnknownVerb;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|verb| verb.name() == text)
            .ok_or_else(|| UnknownVerb(text.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_verb_parses_back_from_its_name() {
        for verb in HelperVerb::ALL {
            assert_eq!(verb.name().parse::<HelperVerb>().ok(), Some(verb));
        }
        assert!("rm".parse::<HelperVerb>().is_err());
    }
}
