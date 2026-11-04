use crownui::{
    kit::use_theme,
    prelude::{Cx, IntoProp, Styled, View, ViewSeq, hstack, spacer, text, vstack},
};

use crate::util::theme::tint;

/// A page title with a muted subtitle on the left and `actions` on the right.
pub fn page_header<Title, Subtitle, Actions>(
    cx: &mut Cx,
    title: Title,
    subtitle: Subtitle,
    actions: Actions,
) -> impl View + use<Title, Subtitle, Actions>
where
    Title: IntoProp<String>,
    Subtitle: IntoProp<String>,
    Actions: ViewSeq,
{
    let theme = use_theme(cx);
    hstack((
        vstack((
            text(title).text_size(24.0).font_bold(),
            text(subtitle)
                .text_size(13.0)
                .text_color(tint(theme, |theme| theme.text.muted)),
        ))
        .gap(2.0),
        spacer(),
        hstack(actions).gap(8.0).items_center(),
    ))
    .items_center()
    .px(24.0)
    .pt(20.0)
    .pb(12.0)
}
