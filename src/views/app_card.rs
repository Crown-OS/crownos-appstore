use crownos_apps_database::{App, AppId};
use crownui::{
    kit::{icon_colored, icons, use_theme},
    prelude::{
        AnyView, Cx, Listeners, Role, Runtime, Semantic, Spring, Styled, View, bind, hstack, show,
        text, vstack,
    },
};

use super::{
    app_icon::{IconFacts, app_icon, theme_color},
    source_label::source_short_name,
};
use crate::{
    state::Store,
    util::theme::{tint, with_alpha},
};

const ICON_SIZE: f32 = 56.0;
const HOVER_WASH: f32 = 0.12;

/// One grid cell: icon, name, summary and where it can come from.
pub fn app_card(cx: &mut Cx, store: Store, position: Option<usize>) -> AnyView {
    let app = position.and_then(|position| {
        store
            .catalog
            .with_untracked(cx, |catalog| catalog.app(position).cloned())
    });
    match app {
        Some(app) => card(cx, store, &app).boxed(),
        None => vstack(()).boxed(),
    }
}

fn card(cx: &mut Cx, store: Store, app: &App) -> impl View + use<> {
    let theme = use_theme(cx);
    let hovered = cx.signal(false);
    let accent = theme_color(app);
    let sources = app
        .sources
        .iter()
        .map(|source| source_short_name(source.kind))
        .collect::<Vec<_>>()
        .join(" · ");
    let id = app.id.clone();
    let installed = move |runtime: &mut Runtime| is_installed(runtime, store, &id);

    let details = vstack((
        text(app.name.clone())
            .text_size(14.0)
            .font_semibold()
            .truncate(),
        text(app.summary.clone())
            .text_size(12.0)
            .line_clamp(2)
            .text_color(tint(theme, |theme| theme.text.muted)),
        text(sources)
            .text_size(11.0)
            .text_color(tint(theme, |theme| theme.text.muted)),
    ))
    .gap(3.0)
    .flex_1()
    .min_w(0.0);
    let badge = show(
        bind(installed),
        || icon_colored(icons::CIRCLE_CHECK, 2.0, |theme| theme.status.success).size(16.0),
        || vstack(()),
    );
    let opened = app.id.clone();
    hstack((
        app_icon(store, IconFacts::new(app), ICON_SIZE),
        details,
        badge,
    ))
    .items_center()
    .gap(12.0)
    .p(12.0)
    .w_full()
    .h_full()
    .rounded(14.0)
    .bg(bind(move |runtime: &mut Runtime| {
        let hovered = hovered.get(runtime);
        theme.with(runtime, |theme| {
            if hovered {
                with_alpha(accent, HOVER_WASH)
            } else {
                theme.surface.raised
            }
        })
    }))
    .border(1.0)
    .border_color(tint(theme, |theme| theme.surface.hairline))
    .transition(Spring::snappy())
    .role(Role::Button)
    .label(app.name.clone())
    .on_pointer_enter(move |cx, _| hovered.set(cx, true))
    .on_pointer_leave(move |cx, _| hovered.set(cx, false))
    .on_click(move |cx, _| store.open_app(cx, opened.clone()))
}

fn is_installed(runtime: &mut Runtime, store: Store, id: &AppId) -> bool {
    runtime.with2(store.catalog, store.sources, |catalog, states| {
        catalog
            .find(id)
            .is_some_and(|app| states.installed_source(app).is_some())
    })
}
