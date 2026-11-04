use crownui::prelude::{AnyView, Cx, View};

use super::{
    app_detail::app_detail, app_grid::app_grid, installed::installed_page, queue_page::queue_page,
    updates::updates_page,
};
use crate::state::{Page, Store};

pub fn build_page(cx: &mut Cx, store: Store, ordinal: usize) -> AnyView {
    match Page::from_ordinal(ordinal) {
        Page::Discover => app_grid(cx, store, None).boxed(),
        Page::Category(category) => app_grid(cx, store, Some(category)).boxed(),
        Page::Installed => installed_page(cx, store).boxed(),
        Page::Updates => updates_page(cx, store).boxed(),
        Page::Queue => queue_page(cx, store).boxed(),
        Page::Detail => app_detail(store).boxed(),
    }
}
