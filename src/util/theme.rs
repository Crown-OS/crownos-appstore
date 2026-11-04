use crownui::{
    kit::Theme,
    prelude::{Color, IntoProp, Signal, bind},
};

/// A color property bound to one slot of `theme`.
pub fn tint(
    theme: Signal<Theme>,
    pick: impl Fn(&Theme) -> Color + 'static,
) -> impl IntoProp<Color> {
    bind(move |runtime| theme.with(runtime, &pick))
}

/// `color` with its translucency, `1 - alpha`, multiplied by `factor`.
pub fn with_translucency_factor(color: Color, factor: f32) -> Color {
    Color {
        a: 1.0 - (1.0 - color.a) * factor,
        ..color
    }
}

pub fn with_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}
