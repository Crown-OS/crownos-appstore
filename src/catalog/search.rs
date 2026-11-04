use crownos_apps_database::App;

/// Lowercased text of one app, built once per catalog so a keystroke only scans.
pub struct Haystack {
    name: String,
    keywords: String,
    summary: String,
}

/// Lower is better.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    NamePrefix,
    NameWord,
    Name,
    Keyword,
    Summary,
}

impl Haystack {
    pub fn new(app: &App) -> Self {
        Self {
            name: app.name.to_lowercase(),
            keywords: app.keywords.join(" ").to_lowercase(),
            summary: app.summary.to_lowercase(),
        }
    }

    /// The worst rank over `terms` (lowercased), or `None` when one is missing.
    pub fn rank(&self, terms: &[&str]) -> Option<Rank> {
        terms
            .iter()
            .map(|term| self.rank_term(term))
            .try_fold(Rank::NamePrefix, |worst, rank| {
                rank.map(|rank| worst.max(rank))
            })
    }

    fn rank_term(&self, term: &str) -> Option<Rank> {
        if self.name.starts_with(term) {
            Some(Rank::NamePrefix)
        } else if self
            .name
            .split_whitespace()
            .any(|word| word.starts_with(term))
        {
            Some(Rank::NameWord)
        } else if self.name.contains(term) {
            Some(Rank::Name)
        } else if self.keywords.contains(term) {
            Some(Rank::Keyword)
        } else if self.summary.contains(term) {
            Some(Rank::Summary)
        } else {
            None
        }
    }
}
