use crownui::{
    kit::{icon_colored, use_theme},
    prelude::{Styled, View, component, text, vstack},
};

use crate::util::theme::tint;

const ICON_SIZE: f32 = 40.0;

pub fn empty_state(icon: &'static str, title: &'static str, detail: &'static str) -> impl View {
    component(move |cx| {
        let theme = use_theme(cx);
        vstack((
            icon_colored(icon, 1.5, |theme| theme.text.icon_muted).size(ICON_SIZE),
            text(title).text_size(16.0).font_semibold(),
            text(detail)
                .text_size(13.0)
                .text_color(tint(theme, |theme| theme.text.muted)),
        ))
        .items_center()
        .justify_center()
        .gap(8.0)
        .flex_1()
        .p(32.0)
    })
}
