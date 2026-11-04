use crownos_apps_database::{AppId, Category};
use crownui::prelude::{Cx, Runtime};

use super::Store;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Discover,
    Category(Category),
    Installed,
    Updates,
    Queue,
    Detail,
}

const FIRST_CATEGORY: usize = 1;
const AFTER_CATEGORIES: usize = FIRST_CATEGORY + Category::ALL.len();

impl Page {
    pub fn ordinal(self) -> usize {
        match self {
            Self::Discover => 0,
            Self::Category(category) => {
                FIRST_CATEGORY
                    + Category::ALL
                        .iter()
                        .position(|&each| each == category)
                        .unwrap_or(0)
            }
            Self::Installed => AFTER_CATEGORIES,
            Self::Updates => AFTER_CATEGORIES + 1,
            Self::Queue => AFTER_CATEGORIES + 2,
            Self::Detail => AFTER_CATEGORIES + 3,
        }
    }

    pub fn from_ordinal(ordinal: usize) -> Self {
        match ordinal {
            0 => Self::Discover,
            ordinal if ordinal < AFTER_CATEGORIES => {
                Self::Category(Category::ALL[ordinal - FIRST_CATEGORY])
            }
            ordinal if ordinal == AFTER_CATEGORIES => Self::Installed,
            ordinal if ordinal == AFTER_CATEGORIES + 1 => Self::Updates,
            ordinal if ordinal == AFTER_CATEGORIES + 2 => Self::Queue,
            _ => Self::Detail,
        }
    }

    pub fn shows_grid(self) -> bool {
        matches!(self, Self::Discover | Self::Category(_))
    }
}

impl Store {
    pub fn show(self, cx: &mut Cx, page: Page) {
        self.page.set(cx, page.ordinal());
    }

    pub fn open_app(self, cx: &mut Cx, id: AppId) {
        let current = self.page.get(cx);
        if Page::from_ordinal(current) != Page::Detail {
            self.previous_page.set(cx, current);
        }
        self.selected.set(cx, Some(id));
        self.show(cx, Page::Detail);
    }

    pub fn back(self, cx: &mut Cx) {
        let previous = self.previous_page.get(cx);
        self.page.set(cx, previous);
    }

    /// The sidebar row to highlight: a detail page keeps the list it came from lit.
    pub fn highlighted(self, runtime: &mut Runtime) -> usize {
        let page = self.page.get(runtime);
        if Page::from_ordinal(page) == Page::Detail {
            self.previous_page.get(runtime)
        } else {
            page
        }
    }

    /// Typing a search from a page without a grid goes to Discover.
    pub(super) fn follow_search(self, cx: &mut Cx) {
        cx.effect(move |runtime| {
            let searching = self.query.with(runtime, |query| !query.trim().is_empty());
            let page = runtime.untrack(|runtime| self.page.get(runtime));
            if searching && !Page::from_ordinal(page).shows_grid() {
                self.page.set(runtime, Page::Discover.ordinal());
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinals_round_trip() {
        let pages = [
            Page::Discover,
            Page::Installed,
            Page::Updates,
            Page::Queue,
            Page::Detail,
        ]
        .into_iter()
        .chain(Category::ALL.map(Page::Category));
        for page in pages {
            assert_eq!(Page::from_ordinal(page.ordinal()), page);
        }
    }
}
