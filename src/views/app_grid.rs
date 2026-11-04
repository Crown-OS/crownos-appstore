use crownos_apps_database::Category;
use crownui::prelude::{Cx, LazyListOptions, Size, Styled, View, bind, lazy_grid, show, vstack};

use super::{app_card::app_card, catalog_notice::catalog_notice, page_header::page_header};
use crate::{state::Store, util::derived::derived};

const CARD: Size = Size::new(300.0, 92.0);
const GAP: (f32, f32) = (12.0, 12.0);
const PADDING: f32 = 24.0;

/// Every app in `category` (all of them for `None`) that matches the search,
/// as a virtualized grid of cards.
pub fn app_grid(cx: &mut Cx, store: Store, category: Option<Category>) -> impl View + use<> {
    let results = derived(cx, move |runtime| {
        let query = store.query.with(runtime, Clone::clone);
        store
            .catalog
            .with(runtime, |catalog| catalog.matching(&query, category))
    });
    let count = derived(cx, move |runtime| results.with(runtime, Vec::len));
    let revision = cx.signal(0_u64);
    cx.effect(move |runtime| {
        results.with(runtime, |_| ());
        runtime.untrack(|runtime| revision.update(runtime, |revision| *revision += 1));
    });

    let title = category.map_or("Discover", Category::label);
    let subtitle = bind(move |runtime| match count.get(runtime) {
        1 => "1 app".to_owned(),
        count => format!("{count} apps"),
    });
    let grid = move || {
        lazy_grid(count, revision, CARD, GAP, move |cx, index| {
            let position = results.with_untracked(cx, |results| results.get(index).copied());
            app_card(cx, store, position)
        })
        .content_padding(PADDING)
        .flex_1()
    };
    vstack((
        page_header(cx, title, subtitle, ()),
        show(
            bind(move |runtime| count.get(runtime) > 0),
            grid,
            move || catalog_notice(store),
        )
        .flex_col()
        .flex_1()
        .min_h(0.0),
    ))
    .flex_1()
    .min_h(0.0)
}
