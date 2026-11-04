use std::fmt;

use serde::{Deserialize, Serialize};

use crate::CatalogError;

const MAX_LENGTH: usize = 128;

/// A catalog id: also the icon's file name, so it never contains a path separator.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AppId(String);

/// A package name as a package manager spells it; never starts with `-`, so it
/// can't be read as an option.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PackageName(String);

fn is_well_formed(text: &str, allowed: impl Fn(char) -> bool) -> bool {
    text.len() <= MAX_LENGTH
        && text.starts_with(|first: char| first.is_ascii_alphanumeric())
        && text.chars().all(allowed)
}

impl AppId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PackageName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for AppId {
    type Error = CatalogError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        let allowed =
            |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '-');
        if is_well_formed(&text, allowed) {
            Ok(Self(text))
        } else {
            Err(CatalogError::InvalidAppId(text))
        }
    }
}

impl TryFrom<String> for PackageName {
    type Error = CatalogError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        let allowed =
            |c: char| c.is_ascii_alphanumeric() || matches!(c, '@' | '.' | '_' | '+' | '-');
        if is_well_formed(&text, allowed) {
            Ok(Self(text))
        } else {
            Err(CatalogError::InvalidPackageName(text))
        }
    }
}

impl TryFrom<&str> for PackageName {
    type Error = CatalogError;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        Self::try_from(text.to_owned())
    }
}

impl From<AppId> for String {
    fn from(id: AppId) -> Self {
        id.0
    }
}

impl From<PackageName> for String {
    fn from(name: PackageName) -> Self {
        name.0
    }
}

impl fmt::Display for AppId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl fmt::Display for PackageName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_names_reject_options_and_shell_syntax() {
        for bad in ["", "-rf", "--noconfirm", "a;rm", "a b", "a/b", "$(x)"] {
            assert!(PackageName::try_from(bad).is_err(), "{bad:?} accepted");
        }
        for good in [
            "firefox",
            "com.github.tchx84.Flatseal",
            "gtk+3",
            "python-foo_bar",
            "lib32@x",
        ] {
            assert!(PackageName::try_from(good).is_ok(), "{good:?} rejected");
        }
    }

    #[test]
    fn app_ids_are_safe_file_names() {
        assert!(AppId::try_from("visual-studio-code".to_owned()).is_ok());
        for bad in ["Firefox", "../etc", "a/b", "", ".hidden"] {
            assert!(AppId::try_from(bad.to_owned()).is_err(), "{bad:?} accepted");
        }
    }
}
