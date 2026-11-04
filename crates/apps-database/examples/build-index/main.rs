//! `cargo build-index [database-dir] [out-dir]`: validates `apps/*/app.toml`
//! and writes `<out-dir>/v1/{index.bin.zst, icons/<id>.png}`.

mod icon;
mod manifest;

use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use crownos_apps_database::{INDEX_FILE, Index, SCHEMA_PATH, validate};

const ICON_DIRECTORY: &str = "icons";
const DEFAULT_DATABASE: &str = "database";
const DEFAULT_OUTPUT: &str = "database/dist";

fn main() -> Result<()> {
    let mut arguments = std::env::args_os().skip(1).map(PathBuf::from);
    let catalog = arguments
        .next()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DATABASE));
    let output = arguments
        .next()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT));
    let published = output.join(SCHEMA_PATH);
    let icons = published.join(ICON_DIRECTORY);
    fs::create_dir_all(&icons).with_context(|| format!("create {}", icons.display()))?;

    let mut apps = Vec::new();
    for (mut app, directory) in manifest::load_all(&catalog.join("apps"))? {
        if let Some(source) = app.icon.take() {
            let file = format!("{}.png", app.id);
            icon::publish(&directory.join(&source), &icons.join(&file))
                .with_context(|| format!("icon of {}", app.id))?;
            app.icon = Some(format!("{ICON_DIRECTORY}/{file}"));
        }
        apps.push(app);
    }
    validate(&apps)?;
    apps.sort_unstable_by_key(|app| app.name.to_lowercase());

    let count = apps.len();
    let encoded = Index::new(apps).encode()?;
    fs::write(published.join(INDEX_FILE), &encoded).context("write index")?;
    println!("{count} apps, index {} bytes", encoded.len());
    Ok(())
}
