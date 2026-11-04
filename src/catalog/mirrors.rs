use std::borrow::Cow;

use crownos_apps_database::SCHEMA_PATH;

/// Tried in order; all serve the same `gh-pages` branch.
const MIRRORS: [&str; 3] = [
    "https://crown-os.github.io/crownos-appstore",
    "https://cdn.jsdelivr.net/gh/Crown-OS/crownos-appstore@gh-pages",
    "https://repo.crownos.org/apps",
];

/// Replaces the mirror list with one base URL, e.g. `file:///path/to/dist`.
const MIRROR_OVERRIDE: &str = "CROWNOS_APPSTORE_CATALOG";

fn bases() -> Vec<Cow<'static, str>> {
    match std::env::var(MIRROR_OVERRIDE) {
        Ok(base) if !base.is_empty() => vec![Cow::Owned(base.trim_end_matches('/').to_owned())],
        _ => MIRRORS.into_iter().map(Cow::Borrowed).collect(),
    }
}

/// `relative` (a path inside the published schema directory) on every mirror.
pub fn urls(relative: &str) -> impl Iterator<Item = String> {
    bases()
        .into_iter()
        .map(move |base| format!("{base}/{SCHEMA_PATH}/{relative}"))
}
