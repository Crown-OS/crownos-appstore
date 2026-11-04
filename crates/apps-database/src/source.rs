use serde::{Deserialize, Serialize};

use crate::PackageName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    Pacman,
    Aur,
    Flatpak,
    Snap,
}

impl SourceKind {
    pub const ALL: [Self; 4] = [Self::Pacman, Self::Aur, Self::Flatpak, Self::Snap];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Pacman => "CrownOS Repos",
            Self::Aur => "AUR",
            Self::Flatpak => "Flathub",
            Self::Snap => "Snap Store",
        }
    }

    pub const fn slug(self) -> &'static str {
        match self {
            Self::Pacman => "pacman",
            Self::Aur => "aur",
            Self::Flatpak => "flatpak",
            Self::Snap => "snap",
        }
    }
}

/// One way to install an app. `classic` only matters to snaps that need
/// classic confinement. Every field is always encoded: the binary index is
/// not self-describing, so a skipped field would shift everything after it.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Source {
    pub kind: SourceKind,
    pub package: PackageName,
    #[serde(default)]
    pub classic: bool,
}
