use serde::{Deserialize, Serialize};

use crate::{AppId, Category, Source, ThemeColor};

/// One catalog entry. `sources` are in order of preference. `icon` is a path
/// relative to the manifest in `app.toml` and relative to the index in the
/// published index.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct App {
    pub id: AppId,
    pub name: String,
    pub summary: String,
    pub description: String,
    pub category: Category,
    pub theme_color: ThemeColor,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    pub sources: Vec<Source>,
}
