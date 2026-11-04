use crownos_apps_database::SourceKind;

pub fn source_name(kind: SourceKind) -> &'static str {
    kind.label()
}

/// The compact form, for cards.
pub fn source_short_name(kind: SourceKind) -> &'static str {
    match kind {
        SourceKind::Pacman => "Repo",
        SourceKind::Aur => "AUR",
        SourceKind::Flatpak => "Flathub",
        SourceKind::Snap => "Snap",
    }
}
