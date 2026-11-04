//! The CrownOS App Store catalog.
//!
//! Contributors describe each app in `database/apps/<id>/app.toml`; `cargo build-index`
//! validates them and publishes one compressed [`Index`] that the store
//! downloads.

mod app;
mod category;
mod error;
mod identifier;
mod index;
mod source;
mod theme_color;
mod validate;

pub use app::App;
pub use category::Category;
pub use error::CatalogError;
pub use identifier::{AppId, PackageName};
pub use index::{INDEX_FILE, Index, SCHEMA_PATH, SCHEMA_VERSION};
pub use source::{Source, SourceKind};
pub use theme_color::ThemeColor;
pub use validate::validate;
