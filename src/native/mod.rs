mod app;
mod bootstrap;
mod browser;
mod commands;
mod lifecycle;
mod presentation;
mod shell;
mod theme;
mod view;

gpui::actions!(kimini, [FocusNext, FocusPrevious]);

pub fn run() {
    shell::run();
}
