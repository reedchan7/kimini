//! Legacy Kimi Code Web shell.
//!
//! Single-site shell for Kimi Code Web: one window, one system WebView,
//! loopback-only navigation. Rendering, GPU compositing, fonts and IME are
//! all delegated to the OS web engine; the Rust host stays a thin shell.
//!
//! Launched with no URL, it discovers (or starts) the local kimi daemon and
//! connects with the persisted token — zero configuration (see `daemon`).

use std::cell::Cell;
#[cfg(target_os = "macos")]
use std::cell::RefCell;
use std::env;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
#[cfg(target_os = "linux")]
use tao::platform::unix::WindowExtUnix;
use tao::window::{WindowBuilder, WindowId};
#[cfg(target_os = "linux")]
use wry::WebViewBuilderExtUnix;
use wry::{NewWindowResponse, WebViewBuilder};

use crate::daemon;
#[cfg(target_os = "macos")]
use crate::i18n;
use crate::i18n::Lang;
#[cfg(target_os = "macos")]
use crate::updater::{LATEST_RELEASE_URL, Updater};

use super::navigation::{explicit_url, is_loopback, origin_for_log};
use super::pages::launch_html;
#[cfg(target_os = "macos")]
use super::{menu, settings};

const APP_TITLE: &str = "Kimini";

/// Stable identifier for the persistent WKWebsiteDataStore (macOS 14+).
/// Keeps localStorage — including the kimi-web server credential — across
/// launches of a bare, unbundled binary.
#[cfg(target_os = "macos")]
const DATA_STORE_ID: [u8; 16] = *b"kimini-data-v001";

pub(super) enum UserEvent {
    SetTitle(String),
    /// Load a loopback URL in the main webview (discovery result or manual connect).
    Navigate(String),
    /// Update the launch page's status line (key understood by `launch_html`).
    LaunchStatus(&'static str),
    /// A page finished loading — replay the last launch status, which may have
    /// been sent before the launch page's script was ready.
    PageLoaded,
    /// IPC message posted by the main webview (the launch page's manual connect).
    MainIpc(String),
    #[cfg(target_os = "macos")]
    Menu(muda::MenuId),
    #[cfg(target_os = "macos")]
    PrefsMessage(String),
}

/// External links are handed to the system browser; the shell never follows them.
fn open_external(raw: &str) {
    if let Err(error) = open::that_detached(raw) {
        eprintln!("kimini: failed to open external URL: {error}");
    }
}

#[cfg(target_os = "macos")]
fn apply_language(
    next: Lang,
    lang_cell: &Cell<Lang>,
    menu_slot: &RefCell<Option<muda::Menu>>,
    settings_slot: &RefCell<Option<settings::SettingsWindow>>,
) {
    if next == lang_cell.get() {
        return;
    }
    lang_cell.set(next);
    if let Err(e) = i18n::save_preference(next) {
        eprintln!("kimini: failed to save language preference: {e}");
    }
    eprintln!("kimini: lang={}", next.code());

    *menu_slot.borrow_mut() = None;
    *menu_slot.borrow_mut() = Some(menu::install(&next.strings()));
    settings::refresh(settings_slot, next);
}

pub fn run() -> wry::Result<()> {
    let lang = Lang::resolve();
    #[cfg(target_os = "macos")]
    let strings = lang.strings();
    eprintln!("kimini: lang={}", lang.code());

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    #[cfg(target_os = "macos")]
    let menu_slot: Rc<RefCell<Option<muda::Menu>>> = Rc::new(RefCell::new(None));
    #[cfg(target_os = "macos")]
    {
        *menu_slot.borrow_mut() = Some(menu::install(&strings));
        let menu_proxy = proxy.clone();
        std::thread::spawn(move || {
            while let Ok(event) = muda::MenuEvent::receiver().recv() {
                let _ = menu_proxy.send_event(UserEvent::Menu(event.id));
            }
        });
    }

    let lang_cell = Rc::new(Cell::new(lang));

    #[cfg(target_os = "macos")]
    let settings_slot: Rc<RefCell<Option<settings::SettingsWindow>>> = Rc::new(RefCell::new(None));
    #[cfg(target_os = "macos")]
    let updater = Updater::new();

    let window = WindowBuilder::new()
        .with_title(APP_TITLE)
        .with_inner_size(tao::dpi::LogicalSize::new(1440.0, 900.0))
        .build(&event_loop)
        .expect("create window");
    let main_id: WindowId = window.id();

    let arg = env::args().nth(1);
    let env_url = env::var("KIMINI_URL").ok();
    let explicit = explicit_url(arg.as_deref(), env_url.as_deref());
    match &explicit {
        Some(url) => eprintln!("kimini: loading {}", origin_for_log(url)),
        None => eprintln!("kimini: no URL given — discovering the local kimi daemon"),
    }

    let title_proxy = proxy.clone();
    let ipc_proxy = proxy.clone();
    let load_proxy = proxy.clone();
    let builder = WebViewBuilder::new()
        .with_devtools(cfg!(debug_assertions))
        .with_document_title_changed_handler(move |title| {
            let _ = title_proxy.send_event(UserEvent::SetTitle(title));
        })
        .with_ipc_handler(move |req| {
            let _ = ipc_proxy.send_event(UserEvent::MainIpc(req.body().to_string()));
        })
        .with_on_page_load_handler(move |event, _url| {
            if matches!(event, wry::PageLoadEvent::Finished) {
                let _ = load_proxy.send_event(UserEvent::PageLoaded);
            }
        })
        .with_navigation_handler(|target| {
            if is_loopback(&target) {
                true
            } else {
                open_external(&target);
                false
            }
        })
        .with_new_window_req_handler(|target, _features| {
            if is_loopback(&target) {
                NewWindowResponse::Allow
            } else {
                open_external(&target);
                NewWindowResponse::Deny
            }
        });
    let builder = match &explicit {
        Some(url) => builder.with_url(url),
        None => builder.with_html(launch_html(lang_cell.get())),
    };

    #[cfg(target_os = "macos")]
    let builder = {
        use wry::WebViewBuilderExtDarwin;
        builder.with_data_store_identifier(DATA_STORE_ID)
    };

    #[cfg(target_os = "linux")]
    let webview = builder.build_gtk(window.gtk_window())?;
    #[cfg(not(target_os = "linux"))]
    let webview = builder.build(&window)?;

    // Zero-config: discover (or start) the daemon off the UI thread; the
    // launch page shows progress until `Navigate` arrives. A manual connect
    // sets the stop flag so the loop exits instead of racing the user.
    let discovery_stop = Arc::new(AtomicBool::new(false));
    let last_launch_status: Cell<Option<&'static str>> = Cell::new(None);
    if explicit.is_none() {
        let discovery_proxy = proxy.clone();
        let stop = Arc::clone(&discovery_stop);
        std::thread::spawn(move || {
            let notify = |status: daemon::Status| {
                let _ = discovery_proxy.send_event(UserEvent::LaunchStatus(status.key()));
            };
            if let Some(url) = daemon::discover(&stop, &notify) {
                let _ = discovery_proxy.send_event(UserEvent::Navigate(url));
            }
        });
    }

    event_loop.run(move |event, target, control_flow| {
        #[cfg(not(target_os = "macos"))]
        let _ = target;
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::CloseRequested,
                ..
            } => {
                if window_id == main_id {
                    *control_flow = ControlFlow::Exit;
                } else {
                    #[cfg(target_os = "macos")]
                    {
                        let mut slot = settings_slot.borrow_mut();
                        if slot.as_ref().is_some_and(|s| s.window.id() == window_id) {
                            *slot = None;
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::SetTitle(title)) => {
                #[cfg(debug_assertions)]
                eprintln!("kimini: page title set ({} chars)", title.len());
                window.set_title(if title.is_empty() { APP_TITLE } else { &title });
            }
            Event::UserEvent(UserEvent::Navigate(url)) => {
                discovery_stop.store(true, Ordering::Relaxed);
                last_launch_status.set(None);
                // Log the origin only — the token rides in the fragment.
                eprintln!("kimini: loading {}", origin_for_log(&url));
                if let Err(e) = webview.load_url(&url) {
                    eprintln!("kimini: failed to load URL: {e}");
                }
            }
            Event::UserEvent(UserEvent::LaunchStatus(key)) => {
                last_launch_status.set(Some(key));
                let _ = webview.evaluate_script(&format!(
                    "window.__kiminiStatus && window.__kiminiStatus('{key}')"
                ));
            }
            Event::UserEvent(UserEvent::PageLoaded) => {
                if let Some(key) = last_launch_status.get() {
                    let _ = webview.evaluate_script(&format!(
                        "window.__kiminiStatus && window.__kiminiStatus('{key}')"
                    ));
                }
            }
            Event::UserEvent(UserEvent::MainIpc(msg)) => {
                // Only the launch page posts messages today; a hostile page
                // gains nothing — connect targets are loopback-gated.
                if let Some(raw) = msg.trim().strip_prefix("connect=") {
                    let raw = raw.trim();
                    if is_loopback(raw) {
                        let _ = proxy.send_event(UserEvent::Navigate(raw.to_string()));
                    } else {
                        let _ = webview.evaluate_script(
                            "window.__kiminiStatus && window.__kiminiStatus('invalidUrl')",
                        );
                    }
                }
            }
            #[cfg(target_os = "macos")]
            Event::UserEvent(UserEvent::Menu(id)) => match id.0.as_str() {
                "reload" => {
                    let _ = webview.reload();
                }
                "back" => {
                    let _ = webview.evaluate_script("history.back()");
                }
                "forward" => {
                    let _ = webview.evaluate_script("history.forward()");
                }
                "settings" => {
                    settings::open_or_focus(target, &proxy, lang_cell.get(), &settings_slot);
                }
                "check-for-updates" if !updater.check_now() => {
                    open_external(LATEST_RELEASE_URL);
                }
                "check-for-updates" => {}
                _ => {}
            },
            #[cfg(target_os = "macos")]
            Event::UserEvent(UserEvent::PrefsMessage(msg)) => {
                let next = match msg.trim() {
                    "lang=en" => Some(Lang::En),
                    "lang=zh" => Some(Lang::Zh),
                    _ => None,
                };
                if let Some(next) = next {
                    apply_language(next, &lang_cell, &menu_slot, &settings_slot);
                }
            }
            _ => {}
        }
    });
}
