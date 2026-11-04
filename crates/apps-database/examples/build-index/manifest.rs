use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use crownos_apps_database::App;

const MANIFEST_FILE: &str = "app.toml";

/// Every `<apps>/<id>/app.toml`, with the directory it was read from.
pub fn load_all(apps: &Path) -> Result<Vec<(App, PathBuf)>> {
    let entries = fs::read_dir(apps).with_context(|| format!("read {}", apps.display()))?;
    entries
        .map(|entry| {
            let directory = entry?.path();
            let app = load(&directory)?;
            Ok((app, directory))
        })
        .collect()
}

fn load(directory: &Path) -> Result<App> {
    let path = directory.join(MANIFEST_FILE);
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let app: App = toml::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    let folder = directory.file_name().and_then(|name| name.to_str());
    if folder != Some(app.id.as_str()) {
        bail!("{} must live in apps/{}/", path.display(), app.id);
    }
    Ok(app)
}
