use std::{borrow::Cow, fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::CatalogError;

/// An app's brand color, `0xRRGGBB`. Written `"#rrggbb"` in manifests and as a
/// plain integer in the binary index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ThemeColor(pub u32);

impl ThemeColor {
    pub const fn rgb(self) -> [u8; 3] {
        let [_, red, green, blue] = self.0.to_be_bytes();
        [red, green, blue]
    }
}

impl FromStr for ThemeColor {
    type Err = CatalogError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.strip_prefix('#')
            .filter(|hex| hex.len() == 6)
            .and_then(|hex| u32::from_str_radix(hex, 16).ok())
            .map(Self)
            .ok_or_else(|| CatalogError::InvalidThemeColor(text.to_owned()))
    }
}

impl fmt::Display for ThemeColor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "#{:06x}", self.0)
    }
}

impl Serialize for ThemeColor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            serializer.serialize_u32(self.0)
        }
    }
}

impl<'de> Deserialize<'de> for ThemeColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            let text = Cow::<str>::deserialize(deserializer)?;
            text.parse().map_err(de::Error::custom)
        } else {
            u32::deserialize(deserializer).map(Self)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_prints_hex() {
        let color: ThemeColor = "#FF7139".parse().unwrap_or(ThemeColor(0));
        assert_eq!(color, ThemeColor(0xff7139));
        assert_eq!(color.to_string(), "#ff7139");
        assert_eq!(color.rgb(), [0xff, 0x71, 0x39]);
        for bad in ["ff7139", "#ff71", "#gggggg", "#ff71399"] {
            assert!(bad.parse::<ThemeColor>().is_err(), "{bad:?} accepted");
        }
    }
}
