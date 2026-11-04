use crownos_apps_database::{App, Source};
use crownui::{
    kit::{
        BadgeOptions, ButtonOptions, ButtonVariant, GradientStops, Theme, ToggleGroupOptions,
        ToggleItem, badge, button, icons, progress, provide_theme, section, setting_row, spinner,
        toggle_group, use_theme,
    },
    prelude::{
        AnyView, Color, Cx, Listeners, Runtime, Signal, Styled, View, WHITE, bind, component,
        hstack, keyed, scroll_view, show, spacer, text, vstack,
    },
};

use super::{
    app_icon::{IconFacts, app_icon, theme_color},
    category_icon::category_icon,
    empty_state::empty_state,
    job_status::{active_job, job_fraction, job_last_line},
    source_label::source_name,
};
use crate::{
    queue::{JobId, JobState},
    state::Store,
    util::{
        open_url::open_url,
        theme::{tint, with_alpha},
    },
};

const ICON_SIZE: f32 = 96.0;
const ACTIONS_WIDTH: f32 = 280.0;
const ACCENT_HIGHLIGHT: f32 = 0.25;
const HERO_WASH: f32 = 0.14;
const HERO_BORDER: f32 = 0.3;

/// The selected app's page, in its own colors.
pub fn app_detail(store: Store) -> impl View {
    keyed(
        move |runtime| store.selected.get(runtime),
        move |cx, selected| {
            let app = selected.and_then(|id| {
                store
                    .catalog
                    .with_untracked(cx, |catalog| catalog.find(&id).cloned())
            });
            match app {
                Some(app) => detail(store, app).boxed(),
                None => empty_state(
                    icons::PACKAGE,
                    "No app selected",
                    "Pick an app to see its details.",
                )
                .boxed(),
            }
        },
    )
    .flex_1()
    .min_h(0.0)
}

fn detail(store: Store, app: App) -> impl View {
    component(move |cx| {
        provide_app_accent(cx, theme_color(&app));
        let preferred = preferred_source(cx, store, &app);
        let chosen = cx.signal(preferred);
        scroll_view(
            vstack((
                toolbar(store, app.homepage.clone()),
                hero(cx, store, &app, chosen),
                about(cx, &app),
                information(cx, store, &app),
            ))
            .gap(20.0)
            .p(24.0),
        )
        .flex_1()
    })
}

/// The desktop theme with the app's color as accent, following the desktop's light and dark mode.
fn provide_app_accent(cx: &mut Cx, color: Color) {
    let desktop = use_theme(cx);
    let accent = GradientStops::new(color.lerp(WHITE, ACCENT_HIGHLIGHT), color);
    let initial = desktop.with_untracked(cx, |theme| Theme { accent, ..*theme });
    let themed = provide_theme(cx, initial);
    cx.effect(move |runtime| {
        let theme = desktop.with(runtime, |theme| Theme { accent, ..*theme });
        themed.set(runtime, theme);
    });
}

fn preferred_source(cx: &mut Cx, store: Store, app: &App) -> usize {
    store.sources.with_untracked(cx, |states| {
        let installed = states.installed_source(app).map(|installed| installed.kind);
        let position = |wanted: &dyn Fn(&Source) -> bool| app.sources.iter().position(wanted);
        installed
            .and_then(|kind| position(&|source| source.kind == kind))
            .or_else(|| position(&|source| states.is_ready(source.kind)))
            .unwrap_or_default()
    })
}

fn toolbar(store: Store, homepage: Option<String>) -> impl View {
    let website = homepage.map(|url| {
        button("Website")
            .variant(ButtonVariant::Ghost)
            .leading_icon(icons::EXTERNAL_LINK)
            .on_click(move |_, _| open_url(&url))
    });
    hstack((
        button("Back")
            .variant(ButtonVariant::Ghost)
            .leading_icon(icons::ARROW_LEFT)
            .on_click(move |cx, _| store.back(cx)),
        spacer(),
        website,
    ))
    .items_center()
}

fn hero(cx: &mut Cx, store: Store, app: &App, chosen: Signal<usize>) -> impl View + use<> {
    let theme = use_theme(cx);
    let accent = theme_color(app);
    let license = app
        .license
        .clone()
        .map(|license| badge(license, tint(theme, |theme| theme.status.neutral)));
    let heading = vstack((
        text(app.name.clone()).text_size(28.0).font_bold(),
        text(app.summary.clone())
            .text_size(14.0)
            .text_color(tint(theme, |theme| theme.text.muted)),
        hstack((
            badge(app.category.label(), accent).leading_icon(category_icon(app.category)),
            license,
        ))
        .gap(6.0),
    ))
    .gap(6.0)
    .flex_1()
    .min_w(0.0);
    hstack((
        app_icon(store, IconFacts::new(app), ICON_SIZE),
        heading,
        actions(cx, store, app, chosen),
    ))
    .items_center()
    .gap(20.0)
    .p(24.0)
    .rounded(20.0)
    .bg(with_alpha(accent, HERO_WASH))
    .border(1.0)
    .border_color(with_alpha(accent, HERO_BORDER))
}

#[derive(Clone, Debug, PartialEq)]
enum ActionState {
    Waiting(JobId),
    Working(JobId),
    Installed,
    Available { needs_setup: bool },
}

fn action_state(
    runtime: &mut Runtime,
    store: Store,
    app: &App,
    chosen: Signal<usize>,
) -> ActionState {
    if let Some((id, state)) = active_job(runtime, store, &app.id) {
        return if state == JobState::Waiting {
            ActionState::Waiting(id)
        } else {
            ActionState::Working(id)
        };
    }
    let index = chosen.get(runtime);
    store
        .sources
        .with(runtime, |states| match states.installed_source(app) {
            Some(_) => ActionState::Installed,
            None => ActionState::Available {
                needs_setup: app
                    .sources
                    .get(index)
                    .is_some_and(|source| states.get(source.kind).ready == Some(false)),
            },
        })
}

fn actions(cx: &mut Cx, store: Store, app: &App, chosen: Signal<usize>) -> impl View + use<> {
    let theme = use_theme(cx);
    let picker = (app.sources.len() > 1).then(|| {
        let items = app
            .sources
            .iter()
            .map(|source| ToggleItem::text(source_name(source.kind)));
        toggle_group(items.collect::<Vec<_>>(), chosen).stretch()
    });
    let observed = app.clone();
    let acted = app.clone();
    let button_area = keyed(
        move |runtime| action_state(runtime, store, &observed, chosen),
        move |cx, state| action_view(cx, store, &acted, chosen, state),
    );
    let described = app.clone();
    let status = text(bind(move |runtime| {
        source_status(runtime, store, &described, chosen)
    }))
    .text_size(12.0)
    .text_color(tint(theme, |theme| theme.text.muted));
    vstack((picker, button_area, status))
        .gap(10.0)
        .w(ACTIONS_WIDTH)
        .shrink(0.0)
}

fn action_view(
    cx: &mut Cx,
    store: Store,
    app: &App,
    chosen: Signal<usize>,
    state: ActionState,
) -> AnyView {
    match state {
        ActionState::Working(id) => working(cx, store, id).boxed(),
        ActionState::Waiting(id) => hstack((
            text("Waiting in queue…").text_size(13.0).flex_1(),
            button("Cancel")
                .variant(ButtonVariant::Secondary)
                .on_click(move |cx, _| store.cancel(cx, id)),
        ))
        .items_center()
        .boxed(),
        ActionState::Installed => {
            let app = app.clone();
            button("Remove")
                .variant(ButtonVariant::Destructive)
                .leading_icon(icons::TRASH_2)
                .stretch()
                .on_click(move |cx, _| store.remove(cx, &app))
                .boxed()
        }
        ActionState::Available { needs_setup } => {
            let app = app.clone();
            let label = if needs_setup {
                "Set Up & Install"
            } else {
                "Install"
            };
            button(label)
                .variant(ButtonVariant::Primary)
                .leading_icon(icons::DOWNLOAD)
                .stretch()
                .on_click(move |cx, _| {
                    let index = chosen.get(cx);
                    store.install(cx, &app, index);
                })
                .boxed()
        }
    }
}

fn working(cx: &mut Cx, store: Store, id: JobId) -> impl View + use<> {
    let theme = use_theme(cx);
    let started = move |runtime: &mut Runtime| job_fraction(runtime, store, id) > 0.0;
    vstack((
        show(
            bind(started),
            move || progress(bind(move |runtime| job_fraction(runtime, store, id))),
            || hstack(spinner()).justify_center(),
        ),
        text(bind(move |runtime| job_last_line(runtime, store, id)))
            .text_size(11.0)
            .truncate()
            .text_color(tint(theme, |theme| theme.text.muted)),
    ))
    .gap(6.0)
}

fn source_status(runtime: &mut Runtime, store: Store, app: &App, chosen: Signal<usize>) -> String {
    let index = chosen.get(runtime);
    store.sources.with(runtime, |states| {
        if let Some(installed) = states.installed_source(app) {
            return format!("Installed from {}", source_name(installed.kind));
        }
        let Some(source) = app.sources.get(index) else {
            return String::new();
        };
        match states.get(source.kind).ready {
            Some(false) => format!("{} will be set up first", source_name(source.kind)),
            _ => format!("{} · {}", source_name(source.kind), source.package),
        }
    })
}

fn about(cx: &mut Cx, app: &App) -> impl View + use<> {
    let theme = use_theme(cx);
    vstack((
        text("About").text_size(16.0).font_semibold(),
        text(app.description.clone())
            .text_size(14.0)
            .leading(1.5)
            .text_color(tint(theme, |theme| theme.text.body)),
    ))
    .gap(8.0)
}

fn information(cx: &mut Cx, store: Store, app: &App) -> impl View + use<> {
    let theme = use_theme(cx);
    let value = move |content: String| {
        text(content)
            .text_size(13.0)
            .text_color(tint(theme, |theme| theme.text.muted))
    };
    let rows: Vec<AnyView> = [
        Some(("Category", app.category.label().to_owned())),
        app.license.clone().map(|license| ("License", license)),
        app.homepage.clone().map(|homepage| ("Website", homepage)),
    ]
    .into_iter()
    .flatten()
    .map(|(label, content)| setting_row(label, value(content)).boxed())
    .chain(app.sources.iter().map(|source| {
        let package = source.package.clone();
        let kind = source.kind;
        let state = text(bind(move |runtime| {
            let installed = store.sources.with(runtime, |states| {
                states.get(kind).installed.contains(package.as_str())
            });
            if installed {
                format!("{package} · installed")
            } else {
                package.to_string()
            }
        }))
        .text_size(13.0)
        .text_color(tint(theme, |theme| theme.text.muted));
        setting_row(source_name(kind), state).boxed()
    }))
    .collect();
    section("Information", rows)
}
