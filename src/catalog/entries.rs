use crownos_apps_database::{App, AppId, Category, Index};

use super::search::Haystack;

/// Every app of the downloaded index, with its search text.
#[derive(Default)]
pub struct Catalog {
    apps: Vec<App>,
    haystacks: Vec<Haystack>,
}

impl Catalog {
    pub fn new(index: Index) -> Self {
        let haystacks = index.apps.iter().map(Haystack::new).collect();
        Self {
            apps: index.apps,
            haystacks,
        }
    }

    pub fn app(&self, position: usize) -> Option<&App> {
        self.apps.get(position)
    }

    pub fn apps(&self) -> &[App] {
        &self.apps
    }

    pub fn find(&self, id: &AppId) -> Option<&App> {
        self.apps.iter().find(|app| app.id == *id)
    }

    /// Positions of the apps matching `query` within `category`, best first;
    /// an empty query keeps the catalog's alphabetical order.
    pub fn matching(&self, query: &str, category: Option<Category>) -> Vec<usize> {
        let query = query.to_lowercase();
        let terms: Vec<&str> = query.split_whitespace().collect();
        let mut ranked: Vec<_> = self
            .apps
            .iter()
            .zip(&self.haystacks)
            .enumerate()
            .filter(|(_, (app, _))| category.is_none_or(|category| app.category == category))
            .filter_map(|(position, (_, haystack))| Some((haystack.rank(&terms)?, position)))
            .collect();
        ranked.sort_by_key(|&(rank, _)| rank);
        ranked.into_iter().map(|(_, position)| position).collect()
    }
}

#[cfg(test)]
mod tests {
    use crownos_apps_database::{CatalogError, PackageName, Source, SourceKind, ThemeColor};

    use super::*;

    fn app(
        name: &str,
        keywords: &[&str],
        summary: &str,
        category: Category,
    ) -> Result<App, CatalogError> {
        Ok(App {
            id: name.to_lowercase().replace(' ', "-").try_into()?,
            name: name.into(),
            summary: summary.into(),
            description: summary.into(),
            category,
            theme_color: ThemeColor(0),
            icon: None,
            homepage: None,
            license: None,
            keywords: keywords.iter().map(|&keyword| keyword.into()).collect(),
            sources: vec![Source {
                kind: SourceKind::Pacman,
                package: PackageName::try_from("x")?,
                classic: false,
            }],
        })
    }

    fn catalog() -> Result<Catalog, CatalogError> {
        Ok(Catalog::new(Index::new(vec![
            app("Blender", &["3d"], "3D creation suite", Category::Graphics)?,
            app("Firefox", &["browser"], "Web browser", Category::Internet)?,
            app(
                "Brave",
                &["browser"],
                "Private browsing",
                Category::Internet,
            )?,
            app(
                "Visual Studio Code",
                &["editor"],
                "Code editor",
                Category::Development,
            )?,
        ])))
    }

    #[test]
    fn ranks_name_prefix_over_word_over_keyword_over_summary() -> Result<(), CatalogError> {
        let catalog = catalog()?;
        assert_eq!(catalog.matching("br", None), [2, 1]);
        assert_eq!(catalog.matching("code", None), [3]);
        assert_eq!(catalog.matching("studio", None), [3]);
        assert_eq!(catalog.matching("suite", None), [0]);
        Ok(())
    }

    #[test]
    fn every_term_must_match_and_category_filters() -> Result<(), CatalogError> {
        let catalog = catalog()?;
        assert_eq!(catalog.matching("web browser", None), [1]);
        assert!(catalog.matching("browser spreadsheet", None).is_empty());
        assert_eq!(catalog.matching("", Some(Category::Internet)), [1, 2]);
        assert_eq!(catalog.matching("  ", None).len(), 4);
        Ok(())
    }
}
