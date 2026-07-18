mod app;
#[cfg(target_os = "macos")]
mod menu;
mod pages;
#[cfg(target_os = "macos")]
mod settings;

pub mod navigation;

pub use app::run;
