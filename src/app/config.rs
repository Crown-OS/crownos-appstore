use crownui::{
    AppConfig,
    kit::{INTER_FAMILY, inter_font},
    prelude::BLACK,
};

const INITIAL_SIZE: (u32, u32) = (1180, 780);
const MIN_SIZE: (u32, u32) = (820, 560);

/// The window the store opens in.
pub fn app_config() -> AppConfig {
    AppConfig::new("App Store")
        .app_id("crownappstore")
        .size(INITIAL_SIZE.0, INITIAL_SIZE.1)
        .min_size(MIN_SIZE.0, MIN_SIZE.1)
        .clear_color(BLACK)
        .mcp(cfg!(debug_assertions))
        .font(inter_font().to_vec(), Some(INTER_FAMILY))
}
