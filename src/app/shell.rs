use crownui::{
    kit::{
        INTER_FAMILY, TextInputOptions, sidebar_search, sidebar_with_header, slide_transition,
        toast_host, use_theme,
    },
    prelude::{Curve, Cx, Styled, View, spacer, vstack, zstack},
};

use crate::{
    state::Store,
    util::theme::{tint, with_translucency_factor},
    views::{build_page, sidebar_menus},
};

const THEME_FADE_MILLIS: u64 = 240;
const CHROME_TRANSLUCENCY: f32 = 0.5;

/// Sidebar, search and the current page over a backdrop in the theme's surface color.
pub fn shell(cx: &mut Cx, store: Store) -> impl View + use<> {
    let theme = use_theme(cx);
    let backdrop = vstack(())
        .w_full()
        .h_full()
        .bg(tint(theme, |theme| {
            with_translucency_factor(theme.surface.bg, CHROME_TRANSLUCENCY)
        }))
        .transition(Curve::ease_in_out(THEME_FADE_MILLIS));
    let content = slide_transition(store.page, move |cx, ordinal| {
        build_page(cx, store, ordinal)
    })
    .flex_1();
    let search = sidebar_search(store.query).placeholder("Search apps");
    let window = sidebar_with_header(sidebar_menus(store), (spacer(), search, spacer()), content);
    zstack((backdrop, toast_host(window).w_full().h_full())).font_family(INTER_FAMILY)
}
