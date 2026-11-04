use crownui::{
    kit::{ButtonOptions, ButtonVariant, CalloutOptions, Tone, button, callout, icons, spinner},
    prelude::{Listeners, Styled, View, keyed, vstack},
};

use super::empty_state::empty_state;
use crate::state::{CatalogStatus, Store};

/// What a grid shows instead of apps: loading, a failed download, or no matches.
pub fn catalog_notice(store: Store) -> impl View {
    keyed(
        move |runtime| store.catalog_status.get(runtime),
        move |_, status| match status {
            CatalogStatus::Loading => vstack(spinner())
                .flex_1()
                .items_center()
                .justify_center()
                .boxed(),
            CatalogStatus::Failed(error) => failed(store, error).boxed(),
            CatalogStatus::Ready => empty_state(
                icons::SEARCH,
                "No apps found",
                "Try a different search or category.",
            )
            .boxed(),
        },
    )
    .flex_1()
}

fn failed(store: Store, error: String) -> impl View {
    vstack((
        callout(Tone::Warning, "The app catalog couldn't be downloaded")
            .body(error)
            .icon(icons::CLOUD_OFF),
        button("Try again")
            .variant(ButtonVariant::Secondary)
            .leading_icon(icons::REFRESH_CW)
            .on_click(move |cx, _| store.sync_catalog(cx)),
    ))
    .gap(12.0)
    .p(32.0)
    .items_center()
    .flex_1()
}
