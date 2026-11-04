use crownconfig::load_appearance;
use crownui::{
    kit::{WindowRadius, provide_configured_theme, provide_window_radius, use_toasts},
    prelude::{View, component},
};

use super::shell::shell;
use crate::state::Store;

/// The window. The store, the theme and the queue worker live in this
/// component's scope, so they last exactly as long as the window.
pub fn root() -> impl View {
    component(|cx| {
        provide_configured_theme(cx);
        provide_window_radius(cx, WindowRadius(f32::from(load_appearance().border_radius)));
        let toasts = use_toasts(cx);
        let store = Store::new(cx, toasts);
        store.start(cx);
        shell(cx, store)
    })
}
