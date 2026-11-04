use crownos_apps_database::{App, AppId};
use crownui::prelude::{Color, Size, Styled, View, WHITE, cover_image, keyed, text, vstack};

use crate::state::{IconKey, Store};

/// Corner radius as a share of the icon's size, for monograms and rounded photos alike.
const CORNER_SHARE: f32 = 0.22;
const MONOGRAM_SHARE: f32 = 0.45;

/// What an icon needs from an app, cloned once when a view is built.
#[derive(Clone)]
pub struct IconFacts {
    id: AppId,
    initial: String,
    color: Color,
    icon: Option<String>,
}

impl IconFacts {
    pub fn new(app: &App) -> Self {
        Self {
            id: app.id.clone(),
            initial: app
                .name
                .chars()
                .next()
                .map(String::from)
                .unwrap_or_default(),
            color: theme_color(app),
            icon: app.icon.clone(),
        }
    }
}

pub fn theme_color(app: &App) -> Color {
    let [red, green, blue] = app.theme_color.rgb();
    Color::from_rgba8(red, green, blue, u8::MAX)
}

/// The app's icon `size` logical pixels square; a monogram in its theme color
/// stands in until (or unless) the icon is decoded.
pub fn app_icon(store: Store, facts: IconFacts, size: f32) -> impl View {
    let key = IconKey::new(facts.id.clone(), size);
    let lookup = key.clone();
    keyed(
        move |runtime| {
            store
                .icons
                .with(runtime, |icons| icons.get(&lookup).copied())
        },
        move |cx, image| match image {
            Some(image) => cover_image(image, Size::new(size, size), size * CORNER_SHARE)
                .size(size)
                .boxed(),
            None => {
                if let Some(relative) = facts.icon.clone() {
                    store.request_icon(cx, key.clone(), relative);
                }
                monogram(&facts, size).boxed()
            }
        },
    )
    .size(size)
}

fn monogram(facts: &IconFacts, size: f32) -> impl View + use<> {
    vstack(
        text(facts.initial.clone())
            .text_size(size * MONOGRAM_SHARE)
            .font_semibold()
            .text_color(WHITE),
    )
    .size(size)
    .items_center()
    .justify_center()
    .rounded(size * CORNER_SHARE)
    .bg(facts.color)
}
