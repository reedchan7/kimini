use std::collections::HashSet;

use gpui::{AppContext, Context, Entity, ScrollHandle, Subscription, Window};
use gpui_component::input::{InputEvent, InputState};

use crate::api::{EventSocket, KimiClient};
use crate::daemon::Connection;
use crate::i18n::{Lang, Strings};
use crate::model::AppModel;
use crate::protocol::ModelCatalogItem;

use super::attachment::Attachments;
use super::auth::AuthState;
use super::browser::BrowserPane;
use super::draft::ComposerDrafts;
use super::files::WorkspaceFiles;
use super::goals::GoalUiState;
use super::presentation::Transcript;
use super::prompt_queue::PromptQueues;
use super::question::QuestionDrafts;
use super::session_list::SessionList;
use super::side_chat::SideChats;
use super::skills::SkillCatalogState;
use super::tasks::TaskRosters;
use super::terminal::{LocalTerminalHost, Terminals};

#[derive(Debug, Clone)]
pub(super) enum LoadState {
    Connecting,
    Ready,
    Working(String),
    Failed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum UtilityPanel {
    Thinking,
    Tasks,
    Auth,
    Files,
    Skills,
    SideChat,
    Terminal,
}

pub(super) struct Shell {
    pub(super) lang: Lang,
    pub(super) strings: Strings,
    pub(super) composer: Entity<InputState>,
    pub(super) session_search: Entity<InputState>,
    pub(super) file_search: Entity<InputState>,
    pub(super) rename_editor: Entity<InputState>,
    pub(super) browser_address: Entity<InputState>,
    pub(super) side_chat_input: Entity<InputState>,
    pub(super) terminal_input: Entity<InputState>,
    pub(super) browser: Option<Entity<BrowserPane>>,
    pub(super) browser_error: Option<String>,
    pub(super) state: LoadState,
    pub(super) model: AppModel,
    pub(super) models: Vec<ModelCatalogItem>,
    pub(super) auth: AuthState,
    pub(super) transcript: Transcript,
    pub(super) session_list: SessionList,
    pub(super) expanded_tools: HashSet<String>,
    pub(super) preview_thinking: Option<String>,
    pub(super) client: Option<KimiClient>,
    pub(super) connection: Option<Connection>,
    pub(super) socket: Option<EventSocket>,
    pub(super) socket_generation: u64,
    pub(super) bootstrap_generation: u64,
    pub(super) snapshot_generation: u64,
    pub(super) sessions_loading: bool,
    pub(super) archives_loading: bool,
    pub(super) show_archived: bool,
    pub(super) history_loading: bool,
    pub(super) renaming_session: bool,
    pub(super) composer_session_id: Option<String>,
    pub(super) drafts: ComposerDrafts,
    pub(super) prompt_queues: PromptQueues,
    pub(super) tasks: TaskRosters,
    pub(super) files: WorkspaceFiles,
    pub(super) skills: SkillCatalogState,
    pub(super) goals: GoalUiState,
    pub(super) side_chats: SideChats,
    pub(super) terminals: Terminals,
    pub(super) local_terminals: LocalTerminalHost,
    pub(super) terminal_scroll: ScrollHandle,
    pub(super) utility_panel: Option<UtilityPanel>,
    pub(super) tasks_loading: bool,
    pub(super) task_error: Option<String>,
    pub(super) task_request_generation: u64,
    pub(super) task_poll_generation: u64,
    pub(super) task_poll_scheduled: bool,
    pub(super) skill_request_generation: u64,
    pub(super) attachments: Attachments,
    pub(super) question_drafts: QuestionDrafts,
    _subscriptions: Vec<Subscription>,
}

impl Shell {
    pub(super) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let lang = Lang::resolve();
        let strings = lang.strings();
        let startup_browser_url = std::env::var("KIMINI_BROWSER_URL").ok();
        let initial_browser_address = startup_browser_url
            .clone()
            .unwrap_or_else(|| "about:blank".into());
        let composer = cx.new(|cx| composer_state(window, cx, strings.native.ask_placeholder));
        let session_search = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(strings.native.search_sessions)
                .default_value("")
        });
        let file_search = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(strings.native.search_files)
                .default_value("")
        });
        let rename_editor = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(strings.native.rename_session)
                .default_value("")
        });
        let browser_address = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(strings.native.browser_address)
                .default_value(initial_browser_address.clone())
        });
        let side_chat_input = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .auto_grow(1, 5)
                .submit_on_enter(true)
                .placeholder(strings.native.side_chat_placeholder)
        });
        let terminal_input = cx.new(|cx| {
            InputState::new(window, cx)
                .submit_on_enter(true)
                .placeholder(strings.native.terminal_placeholder)
        });
        let subscriptions = vec![
            cx.subscribe_in(
                &composer,
                window,
                |this, _, event: &InputEvent, window, cx| {
                    if matches!(event, InputEvent::Change) {
                        this.store_active_composer_draft(cx);
                    }
                    if matches!(event, InputEvent::PressEnter { shift: false, .. }) {
                        this.submit(window, cx);
                    }
                },
            ),
            cx.subscribe(&session_search, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            }),
            cx.subscribe(&file_search, |this, input, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) && input.read(cx).value().trim().is_empty() {
                    this.files.clear_search();
                    cx.notify();
                }
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.search_workspace_files(cx);
                }
            }),
            cx.subscribe_in(
                &browser_address,
                window,
                |this, _, event: &InputEvent, window, cx| {
                    if matches!(event, InputEvent::PressEnter { .. }) {
                        this.navigate_browser(window, cx);
                    }
                },
            ),
            cx.subscribe_in(
                &rename_editor,
                window,
                |this, _, event: &InputEvent, window, cx| {
                    if matches!(event, InputEvent::PressEnter { shift: false, .. }) {
                        this.commit_session_rename(window, cx);
                    }
                },
            ),
            cx.subscribe_in(
                &side_chat_input,
                window,
                |this, _, event: &InputEvent, window, cx| {
                    if matches!(event, InputEvent::PressEnter { shift: false, .. }) {
                        this.send_side_chat_prompt(window, cx);
                    }
                },
            ),
            cx.subscribe_in(
                &terminal_input,
                window,
                |this, _, event: &InputEvent, window, cx| {
                    if matches!(event, InputEvent::PressEnter { shift: false, .. }) {
                        this.send_terminal_command(window, cx);
                    }
                },
            ),
        ];
        let mut shell = Self {
            lang,
            strings,
            composer,
            session_search,
            file_search,
            rename_editor,
            browser_address,
            side_chat_input,
            terminal_input,
            browser: None,
            browser_error: None,
            state: LoadState::Connecting,
            model: AppModel::default(),
            models: Vec::new(),
            auth: AuthState::default(),
            transcript: Transcript::default(),
            session_list: SessionList::default(),
            expanded_tools: HashSet::new(),
            preview_thinking: None,
            client: None,
            connection: None,
            socket: None,
            socket_generation: 0,
            bootstrap_generation: 0,
            snapshot_generation: 0,
            sessions_loading: false,
            archives_loading: false,
            show_archived: false,
            history_loading: false,
            renaming_session: false,
            composer_session_id: None,
            drafts: ComposerDrafts::default(),
            prompt_queues: PromptQueues::default(),
            tasks: TaskRosters::default(),
            files: WorkspaceFiles::default(),
            skills: SkillCatalogState::default(),
            goals: GoalUiState::default(),
            side_chats: SideChats::default(),
            terminals: Terminals::default(),
            local_terminals: LocalTerminalHost::default(),
            terminal_scroll: ScrollHandle::new(),
            utility_panel: None,
            tasks_loading: false,
            task_error: None,
            task_request_generation: 0,
            task_poll_generation: 0,
            task_poll_scheduled: false,
            skill_request_generation: 0,
            attachments: Attachments::default(),
            question_drafts: QuestionDrafts::default(),
            _subscriptions: subscriptions,
        };
        shell.start_local_terminal_events(cx);
        if startup_browser_url.is_some() {
            shell.open_browser(window, cx);
        } else {
            shell
                .composer
                .update(cx, |input, cx| input.focus(window, cx));
        }
        shell.start_bootstrap(cx);
        shell
    }
}

fn composer_state(
    window: &mut Window,
    cx: &mut Context<InputState>,
    placeholder: &'static str,
) -> InputState {
    InputState::new(window, cx)
        .multi_line(true)
        .auto_grow(2, 8)
        .submit_on_enter(true)
        .placeholder(placeholder)
}
