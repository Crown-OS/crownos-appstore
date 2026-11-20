mod config;
mod root;
mod shell;

pub use config::app_config;
pub use root::root;

#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod tests;
