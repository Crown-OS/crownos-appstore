use crownos_apps_database::SourceKind;
use crownui::{
    kit::{ButtonOptions, ButtonVariant, button, icons, spinner, use_theme},
    prelude::{
        AnyView, Cx, Listeners, Runtime, Styled, View, bind, for_each, hstack, scroll_view, show,
        spacer, text, vstack,
    },
};

use super::{empty_state::empty_state, page_header::page_header, source_label::source_name};
use crate::{
    queue::Action,
    source::PendingUpdate,
    state::Store,
    util::{derived::derived, theme::tint},
};

const PACMAN_NOTE: &str =
    "Arch updates the whole system at once, so this installs every pending repository update.";

pub fn updates_page(cx: &mut Cx, store: Store) -> impl View + use<> {
    let total = derived(cx, move |runtime| {
        store.sources.with(runtime, |states| states.update_count())
    });
    let subtitle = bind(move |runtime| match total.get(runtime) {
        0 => "Everything is up to date".to_owned(),
        1 => "1 update available".to_owned(),
        count => format!("{count} updates available"),
    });
    let actions = (
        button("Check Again")
            .variant(ButtonVariant::Secondary)
            .leading_icon(icons::REFRESH_CW)
            .on_click(move |cx, _| store.check_updates(cx)),
        show(
            bind(move |runtime| total.get(runtime) > 0),
            move || {
                button("Update All")
                    .variant(ButtonVariant::Primary)
                    .leading_icon(icons::DOWNLOAD)
                    .on_click(move |cx, _| store.update_everything(cx))
            },
            || vstack(()),
        ),
    );
    let groups = move || {
        let groups: Vec<AnyView> = SourceKind::ALL
            .into_iter()
            .map(|kind| update_group(store, kind).boxed())
            .collect();
        scroll_view(vstack(groups).gap(16.0).px(24.0).pb(24.0)).flex_1()
    };
    vstack((
        page_header(cx, "Updates", subtitle, actions),
        show(bind(move |runtime| total.get(runtime) > 0), groups, || {
            empty_state(
                icons::CIRCLE_CHECK,
                "Everything is up to date",
                "CrownOS checks every source when the store opens.",
            )
        })
        .flex_col()
        .flex_1()
        .min_h(0.0),
    ))
    .flex_1()
    .min_h(0.0)
}

fn update_group(store: Store, kind: SourceKind) -> impl View {
    let has_updates = move |runtime: &mut Runtime| {
        store
            .sources
            .with(runtime, |states| !states.get(kind).updates.is_empty())
    };
    show(
        bind(has_updates),
        move || group_body(store, kind),
        || vstack(()),
    )
}

fn group_body(store: Store, kind: SourceKind) -> impl View {
    crownui::prelude::component(move |cx| {
        let theme = use_theme(cx);
        let updates = derived(cx, move |runtime| {
            store
                .sources
                .with(runtime, |states| states.get(kind).updates.clone())
        });
        let updating = move |runtime: &mut Runtime| {
            store.jobs.with(runtime, |jobs| {
                jobs.iter().any(|entry| {
                    entry.job.kind == kind
                        && entry.job.action == Action::UpdateAll
                        && entry.state.is_active()
                })
            })
        };
        let heading = hstack((
            text(source_name(kind)).text_size(16.0).font_semibold(),
            text(bind(move |runtime| {
                updates.with(runtime, Vec::len).to_string()
            }))
            .text_size(13.0)
            .text_color(tint(theme, |theme| theme.text.muted)),
            spacer(),
            show(bind(updating), spinner, move || {
                button("Update")
                    .variant(ButtonVariant::Tinted)
                    .on_click(move |cx, _| store.update_source(cx, kind))
            }),
        ))
        .items_center()
        .gap(8.0);
        let note = (kind == SourceKind::Pacman).then(|| {
            text(PACMAN_NOTE)
                .text_size(12.0)
                .text_color(tint(theme, |theme| theme.text.muted))
        });
        let rows = for_each(
            updates,
            |update: &PendingUpdate| update.package.clone(),
            move |cx, update| {
                let theme = use_theme(cx);
                let PendingUpdate {
                    package, detail, ..
                } = update.with_untracked(cx, Clone::clone);
                hstack((
                    text(package)
                        .text_size(13.0)
                        .font_medium()
                        .flex_1()
                        .truncate(),
                    text(detail)
                        .text_size(12.0)
                        .text_color(tint(theme, |theme| theme.text.muted)),
                ))
                .gap(12.0)
                .py(6.0)
            },
        );
        vstack((heading, note, vstack(rows)))
            .gap(8.0)
            .p(16.0)
            .rounded(14.0)
            .bg(tint(theme, |theme| theme.surface.raised))
    })
}
