use crownos_apps_database::{App, AppId, SourceKind};
use crownui::{
    kit::{ButtonOptions, ButtonVariant, button, icons, spinner, use_theme},
    prelude::{
        AnyView, Cx, Listeners, Role, Runtime, Semantic, Signal, Styled, View, bind, for_each,
        hstack, scroll_view, show, text, vstack,
    },
};

use super::{
    app_icon::{IconFacts, app_icon},
    empty_state::empty_state,
    job_status::active_job,
    page_header::page_header,
    source_label::source_name,
};
use crate::{
    state::Store,
    util::{derived::derived, theme::tint},
};

const ICON_SIZE: f32 = 40.0;

#[derive(Clone, Debug, PartialEq)]
struct InstalledRow {
    id: AppId,
    kind: SourceKind,
}

/// Catalog apps found installed by any source, filtered by the search.
pub fn installed_page(cx: &mut Cx, store: Store) -> impl View + use<> {
    let rows = derived(cx, move |runtime| installed_rows(runtime, store));
    let subtitle = bind(move |runtime| match rows.with(runtime, Vec::len) {
        1 => "1 app from the catalog".to_owned(),
        count => format!("{count} apps from the catalog"),
    });
    let list = move || {
        scroll_view(
            vstack(for_each(
                rows,
                |row: &InstalledRow| row.id.clone(),
                move |cx, row| installed_row(cx, store, row),
            ))
            .gap(8.0)
            .px(24.0)
            .pb(24.0),
        )
        .flex_1()
    };
    vstack((
        page_header(cx, "Installed", subtitle, ()),
        show(
            bind(move |runtime| rows.with(runtime, |rows| !rows.is_empty())),
            list,
            || {
                empty_state(
                    icons::PACKAGE,
                    "Nothing installed yet",
                    "Apps you install from the store appear here.",
                )
            },
        )
        .flex_col()
        .flex_1()
        .min_h(0.0),
    ))
    .flex_1()
    .min_h(0.0)
}

fn installed_rows(runtime: &mut Runtime, store: Store) -> Vec<InstalledRow> {
    let query = store.query.with(runtime, |query| query.clone());
    runtime.with2(store.catalog, store.sources, |catalog, states| {
        catalog
            .matching(&query, None)
            .into_iter()
            .filter_map(|position| {
                let app = catalog.app(position)?;
                let source = states.installed_source(app)?;
                Some(InstalledRow {
                    id: app.id.clone(),
                    kind: source.kind,
                })
            })
            .collect()
    })
}

fn installed_row(cx: &mut Cx, store: Store, row: Signal<InstalledRow>) -> AnyView {
    let InstalledRow { id, kind } = row.with_untracked(cx, Clone::clone);
    let Some(app) = store
        .catalog
        .with_untracked(cx, |catalog| catalog.find(&id).cloned())
    else {
        return vstack(()).boxed();
    };
    let theme = use_theme(cx);
    let busy_id = id.clone();
    let busy = move |runtime: &mut Runtime| active_job(runtime, store, &busy_id).is_some();
    let removed = app.clone();
    hstack((
        app_icon(store, IconFacts::new(&app), ICON_SIZE),
        vstack((
            text(app.name.clone()).text_size(14.0).font_semibold(),
            text(format!("From {}", source_name(kind)))
                .text_size(12.0)
                .text_color(tint(theme, |theme| theme.text.muted)),
        ))
        .gap(2.0)
        .flex_1()
        .min_w(0.0),
        show(bind(busy), spinner, move || {
            remove_button(store, removed.clone())
        }),
    ))
    .items_center()
    .gap(12.0)
    .p(12.0)
    .rounded(12.0)
    .bg(tint(theme, |theme| theme.surface.raised))
    .role(Role::ListItem)
    .label(app.name.clone())
    .on_click(move |cx, _| store.open_app(cx, id.clone()))
    .boxed()
}

fn remove_button(store: Store, app: App) -> impl View {
    button("Remove")
        .variant(ButtonVariant::Secondary)
        .leading_icon(icons::TRASH_2)
        .on_click(move |cx, event| {
            event.stop_propagation();
            store.remove(cx, &app);
        })
}
