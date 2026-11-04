use std::collections::HashSet;

use crate::{App, CatalogError};

/// Checks what the types alone can't: unique ids, at least one source, one
/// source per kind and no blank text.
pub fn validate(apps: &[App]) -> Result<(), CatalogError> {
    let mut ids = HashSet::with_capacity(apps.len());
    apps.iter().try_for_each(|app| {
        if !ids.insert(&app.id) {
            return Err(CatalogError::DuplicateApp(app.id.to_string()));
        }
        validate_app(app)
    })
}

fn validate_app(app: &App) -> Result<(), CatalogError> {
    let blank = [
        ("name", &app.name),
        ("summary", &app.summary),
        ("description", &app.description),
    ]
    .into_iter()
    .find(|(_, text)| text.trim().is_empty());
    if let Some((field, _)) = blank {
        return Err(CatalogError::EmptyField {
            app: app.id.to_string(),
            field,
        });
    }
    if app.sources.is_empty() {
        return Err(CatalogError::NoSources(app.id.to_string()));
    }
    let mut kinds = HashSet::with_capacity(app.sources.len());
    match app.sources.iter().find(|source| !kinds.insert(source.kind)) {
        Some(source) => Err(CatalogError::DuplicateSource {
            app: app.id.to_string(),
            kind: source.kind.slug(),
        }),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Category, PackageName, Source, SourceKind, ThemeColor};

    fn app(id: &str, sources: &[SourceKind]) -> Result<App, CatalogError> {
        let sources = sources
            .iter()
            .map(|&kind| {
                Ok(Source {
                    kind,
                    package: PackageName::try_from(id)?,
                    classic: false,
                })
            })
            .collect::<Result<_, CatalogError>>()?;
        Ok(App {
            id: id.to_owned().try_into()?,
            name: id.into(),
            summary: "summary".into(),
            description: "description".into(),
            category: Category::Utilities,
            theme_color: ThemeColor(0),
            icon: None,
            homepage: None,
            license: None,
            keywords: Vec::new(),
            sources,
        })
    }

    #[test]
    fn accepts_a_clean_catalog() -> Result<(), CatalogError> {
        validate(&[
            app("a", &[SourceKind::Pacman])?,
            app("b", &[SourceKind::Snap, SourceKind::Aur])?,
        ])
    }

    #[test]
    fn rejects_duplicates_and_missing_sources() -> Result<(), CatalogError> {
        let twice = [
            app("a", &[SourceKind::Pacman])?,
            app("a", &[SourceKind::Aur])?,
        ];
        assert!(matches!(
            validate(&twice),
            Err(CatalogError::DuplicateApp(_))
        ));
        assert!(matches!(
            validate(&[app("a", &[])?]),
            Err(CatalogError::NoSources(_))
        ));
        let same_kind = [app("a", &[SourceKind::Aur, SourceKind::Aur])?];
        assert!(matches!(
            validate(&same_kind),
            Err(CatalogError::DuplicateSource { .. })
        ));
        Ok(())
    }
}
