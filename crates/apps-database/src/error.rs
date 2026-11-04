use thiserror::Error;

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("invalid app id {0:?}: use lowercase letters, digits, '.' and '-'")]
    InvalidAppId(String),
    #[error("invalid package name {0:?}")]
    InvalidPackageName(String),
    #[error("invalid theme color {0:?}: expected #rrggbb")]
    InvalidThemeColor(String),
    #[error("app {0} is listed twice")]
    DuplicateApp(String),
    #[error("app {0} has no install source")]
    NoSources(String),
    #[error("app {app} lists the {kind} source twice")]
    DuplicateSource { app: String, kind: &'static str },
    #[error("app {app} has an empty {field}")]
    EmptyField { app: String, field: &'static str },
    #[error("index schema {found} is not supported (expected {expected})")]
    UnsupportedSchema { found: u32, expected: u32 },
    #[error("index is corrupt: {0}")]
    Decode(#[from] postcard::Error),
    #[error("index compression failed: {0}")]
    Compression(#[from] std::io::Error),
}
