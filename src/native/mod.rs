mod app;
mod attachment;
mod auth;
mod bootstrap;
mod browser;
mod commands;
mod draft;
mod files;
mod goals;
mod lifecycle;
mod presentation;
mod prompt_queue;
mod prompt_runtime;
mod question;
mod session_list;
mod shell;
mod side_chat;
mod skills;
mod slash;
mod tasks;
mod terminal;
mod theme;
mod view;

gpui::actions!(
    kimini,
    [
        FocusNext,
        FocusPrevious,
        RenameSession,
        ForkSession,
        CompactSession,
        UndoSession,
        ArchiveSession,
        ExportSession,
        SteerPrompt,
        ToggleTasks,
        ToggleFiles,
        ToggleSkills,
        ToggleTerminal
    ]
);

#[derive(Clone, PartialEq, gpui::Action)]
#[action(namespace = kimini, no_json)]
struct SetModel {
    model: String,
}

#[derive(Clone, PartialEq, gpui::Action)]
#[action(namespace = kimini, no_json)]
struct SetThinking {
    effort: String,
}

#[derive(Clone, PartialEq, gpui::Action)]
#[action(namespace = kimini, no_json)]
struct SetPermission {
    mode: String,
}

pub fn run() {
    shell::run();
}
