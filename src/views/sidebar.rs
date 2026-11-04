use crownos_apps_database::Category;
use crownui::{
    kit::{SidebarItemOptions, icons, sidebar_item, sidebar_section},
    prelude::{AnyView, Listeners, Runtime, View, bind},
};

use super::category_icon::category_icon;
use crate::state::{Page, Store};

pub fn sidebar_menus(store: Store) -> Vec<AnyView> {
    let item = move |page: Page, label: &'static str, icon: &'static str| {
        let ordinal = page.ordinal();
        sidebar_item(
            label,
            icon,
            bind(move |runtime| store.highlighted(runtime) == ordinal),
        )
        .on_click(move |cx, _| store.show(cx, page))
    };
    let count = move |read: fn(&mut Runtime, Store) -> usize| {
        bind(move |runtime| match read(runtime, store) {
            0 => String::new(),
            count => count.to_string(),
        })
    };
    let store_section = sidebar_section(
        "Store",
        (
            item(Page::Discover, "Discover", icons::COMPASS),
            item(Page::Installed, "Installed", icons::HARD_DRIVE_DOWNLOAD)
                .trailing_text(count(installed_count)),
            item(Page::Updates, "Updates", icons::REFRESH_CW).trailing_text(count(update_count)),
            item(Page::Queue, "Queue", icons::LIST_ORDERED).trailing_text(count(active_job_count)),
        ),
    );
    let categories: Vec<_> = Category::ALL
        .into_iter()
        .map(|category| {
            item(
                Page::Category(category),
                category.label(),
                category_icon(category),
            )
        })
        .collect();
    vec![
        store_section.boxed(),
        sidebar_section("Categories", categories).boxed(),
    ]
}

fn installed_count(runtime: &mut Runtime, store: Store) -> usize {
    runtime.with2(store.catalog, store.sources, |catalog, states| {
        catalog
            .apps()
            .iter()
            .filter(|app| states.installed_source(app).is_some())
            .count()
    })
}

fn update_count(runtime: &mut Runtime, store: Store) -> usize {
    store.sources.with(runtime, |states| states.update_count())
}

fn active_job_count(runtime: &mut Runtime, store: Store) -> usize {
    store.jobs.with(runtime, |jobs| {
        jobs.iter().filter(|entry| entry.state.is_active()).count()
    })
}
