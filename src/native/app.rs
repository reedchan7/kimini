use gpui::{AppContext, Context, Entity, Subscription, Window};
use gpui_component::input::{InputEvent, InputState};

use crate::api::{EventSocket, KimiClient};
use crate::daemon::Connection;
use crate::model::AppModel;

use super::browser::BrowserPane;
use super::presentation::Transcript;

#[derive(Debug, Clone)]
pub(super) enum LoadState {
    Connecting,
    Ready,
    Working(String),
    Failed(String),
}

pub(super) struct Shell {
    pub(super) composer: Entity<InputState>,
    pub(super) browser_address: Entity<InputState>,
    pub(super) browser: Option<Entity<BrowserPane>>,
    pub(super) browser_error: Option<String>,
    pub(super) state: LoadState,
    pub(super) model: AppModel,
    pub(super) transcript: Transcript,
    pub(super) client: Option<KimiClient>,
    pub(super) connection: Option<Connection>,
    pub(super) socket: Option<EventSocket>,
    pub(super) socket_generation: u64,
    pub(super) snapshot_generation: u64,
    _subscriptions: Vec<Subscription>,
}

impl Shell {
    pub(super) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let startup_browser_url = std::env::var("KIMINI_BROWSER_URL").ok();
        let initial_browser_address = startup_browser_url
            .clone()
            .unwrap_or_else(|| "about:blank".into());
        let composer = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .placeholder("Ask Kimi to work on this project…")
        });
        let browser_address = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Enter a web address")
                .default_value(initial_browser_address.clone())
        });
        let subscriptions = vec![
            cx.subscribe_in(
                &composer,
                window,
                |this, _, event: &InputEvent, window, cx| {
                    if matches!(
                        event,
                        InputEvent::PressEnter {
                            secondary: true,
                            ..
                        }
                    ) {
                        this.submit(window, cx);
                    }
                },
            ),
            cx.subscribe_in(
                &browser_address,
                window,
                |this, _, event: &InputEvent, window, cx| {
                    if matches!(event, InputEvent::PressEnter { .. }) {
                        this.navigate_browser(window, cx);
                    }
                },
            ),
        ];
        let mut shell = Self {
            composer,
            browser_address,
            browser: None,
            browser_error: None,
            state: LoadState::Connecting,
            model: AppModel::default(),
            transcript: Transcript::default(),
            client: None,
            connection: None,
            socket: None,
            socket_generation: 0,
            snapshot_generation: 0,
            _subscriptions: subscriptions,
        };
        if startup_browser_url.is_some() {
            shell.open_browser(window, cx);
        }
        shell.start_bootstrap(cx);
        shell
    }
}
