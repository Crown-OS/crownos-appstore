use serde::{Deserialize, Serialize};

use crate::{App, CatalogError};

/// Bumped on every incompatible format change; it is also the URL path, so
/// old stores keep reading the old index.
pub const SCHEMA_VERSION: u32 = 1;
pub const SCHEMA_PATH: &str = "v1";
pub const INDEX_FILE: &str = "index.bin.zst";

const COMPRESSION_LEVEL: i32 = 19;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Index {
    pub schema_version: u32,
    pub apps: Vec<App>,
}

impl Index {
    pub fn new(apps: Vec<App>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            apps,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, CatalogError> {
        let packed = postcard::to_stdvec(self)?;
        Ok(zstd::encode_all(packed.as_slice(), COMPRESSION_LEVEL)?)
    }

    pub fn decode(compressed: &[u8]) -> Result<Self, CatalogError> {
        let packed = zstd::decode_all(compressed)?;
        let index: Self = postcard::from_bytes(&packed)?;
        if index.schema_version != SCHEMA_VERSION {
            return Err(CatalogError::UnsupportedSchema {
                found: index.schema_version,
                expected: SCHEMA_VERSION,
            });
        }
        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Category, PackageName, Source, SourceKind, ThemeColor};

    fn sample() -> Result<Index, CatalogError> {
        Ok(Index::new(vec![App {
            id: "firefox".to_owned().try_into()?,
            name: "Firefox".into(),
            summary: "Web browser".into(),
            description: "Fast.".into(),
            category: Category::Internet,
            theme_color: ThemeColor(0xff7139),
            icon: Some("icons/firefox.png".into()),
            homepage: None,
            license: Some("MPL-2.0".into()),
            keywords: vec!["browser".into()],
            sources: vec![
                Source {
                    kind: SourceKind::Pacman,
                    package: PackageName::try_from("firefox")?,
                    classic: false,
                },
                Source {
                    kind: SourceKind::Snap,
                    package: PackageName::try_from("firefox")?,
                    classic: true,
                },
            ],
        }]))
    }

    #[test]
    fn round_trips() -> Result<(), CatalogError> {
        let index = sample()?;
        assert_eq!(Index::decode(&index.encode()?)?, index);
        Ok(())
    }

    #[test]
    fn rejects_other_schemas() -> Result<(), CatalogError> {
        let mut index = sample()?;
        index.schema_version = SCHEMA_VERSION + 1;
        assert!(matches!(
            Index::decode(&index.encode()?),
            Err(CatalogError::UnsupportedSchema { .. })
        ));
        Ok(())
    }
}
