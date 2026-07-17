//! Kimini — the lightest way to browse.
//!
//! Single-site shell for Kimi Code Web: one window, one system WebView,
//! loopback-only navigation. Rendering, GPU compositing, fonts and IME are
//! all delegated to the OS web engine; the Rust host stays a thin shell.
//!
//! Launched with no URL, it discovers (or starts) the local kimi daemon and
//! connects with the persisted token — zero configuration (see `daemon`).

mod daemon;
mod i18n;

use std::cell::{Cell, RefCell};
use std::env;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tao::window::{Window, WindowBuilder, WindowId};
use url::Url;
use wry::{NewWindowResponse, WebView, WebViewBuilder};

use i18n::{Lang, Strings, launch_html, settings_html};

const APP_TITLE: &str = "Kimini";

/// Stable identifier for the persistent WKWebsiteDataStore (macOS 14+).
/// Keeps localStorage — including the kimi-web server credential — across
/// launches of a bare, unbundled binary.
#[cfg(target_os = "macos")]
const DATA_STORE_ID: [u8; 16] = *b"kimini-data-v001";

enum UserEvent {
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

/// First CLI argument wins, then `$KIMINI_URL`. `None` means zero-config:
/// discover (or start) the local kimi daemon and connect with its persisted
/// token. An explicit URL skips discovery entirely — useful for a non-default
/// `KIMI_CODE_HOME` or a port-forwarded daemon.
fn explicit_url() -> Option<String> {
    env::args()
        .nth(1)
        .or_else(|| env::var("KIMINI_URL").ok())
        .filter(|u| !u.is_empty())
}

/// Only loopback origins may navigate inside the shell.
fn is_loopback(raw: &str) -> bool {
    let Ok(url) = Url::parse(raw) else {
        return false;
    };
    if url.scheme() == "about" {
        return true; // about:blank
    }
    if !matches!(url.scheme(), "http" | "https") {
        return false;
    }
    match url.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        Some(url::Host::Domain(domain)) => {
            domain.eq_ignore_ascii_case("localhost")
                || domain.to_ascii_lowercase().ends_with(".localhost")
        }
        None => false,
    }
}

/// External links are handed to the system browser; the shell never follows them.
fn open_external(raw: &str) {
    #[cfg(target_os = "macos")]
    let cmd = std::process::Command::new("open").arg(raw).spawn();
    #[cfg(target_os = "linux")]
    let cmd = std::process::Command::new("xdg-open").arg(raw).spawn();
    #[cfg(target_os = "windows")]
    let cmd = std::process::Command::new("cmd")
        .args(["/c", "start", "", raw])
        .spawn();
    let _ = cmd;
}

/// macOS routes Cmd+C/V/A/Z through the main menu. Language lives under
/// **Settings…** (not a top-level Language menu).
#[cfg(target_os = "macos")]
fn install_menu(t: &Strings) -> muda::Menu {
    use muda::{Menu, MenuItem, PredefinedMenuItem, Submenu};

    let menu = Menu::new();
    let app = Submenu::with_id_and_items(
        "app",
        APP_TITLE,
        true,
        &[
            &PredefinedMenuItem::about(Some(t.about), None),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id("settings", t.settings, true, "CmdOrCtrl+,".parse().ok()),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(None),
        ],
    );
    let edit = Submenu::with_id_and_items(
        "edit",
        t.edit,
        true,
        &[
            &PredefinedMenuItem::undo(None),
            &PredefinedMenuItem::redo(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::cut(None),
            &PredefinedMenuItem::copy(None),
            &PredefinedMenuItem::paste(None),
            &PredefinedMenuItem::select_all(None),
        ],
    );
    let navigate = Submenu::with_id_and_items(
        "navigate",
        t.navigate,
        true,
        &[
            &MenuItem::with_id("reload", t.reload, true, "CmdOrCtrl+R".parse().ok()),
            &MenuItem::with_id("back", t.back, true, "CmdOrCtrl+[".parse().ok()),
            &MenuItem::with_id("forward", t.forward, true, "CmdOrCtrl+]".parse().ok()),
        ],
    );
    let window = Submenu::with_id_and_items(
        "window",
        t.window,
        true,
        &[
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::close_window(None),
        ],
    );
    for submenu in [app, edit, navigate, window] {
        menu.append(&submenu.expect("valid menu item"))
            .expect("append submenu");
    }
    menu.init_for_nsapp();
    menu
}

#[cfg(target_os = "macos")]
struct SettingsWindow {
    window: Window,
    webview: WebView,
}

#[cfg(target_os = "macos")]
fn open_or_focus_settings(
    event_loop: &tao::event_loop::EventLoopWindowTarget<UserEvent>,
    proxy: &EventLoopProxy<UserEvent>,
    lang: Lang,
    slot: &RefCell<Option<SettingsWindow>>,
) {
    if let Some(existing) = slot.borrow().as_ref() {
        existing.window.set_focus();
        return;
    }

    let t = lang.strings();
    let window = WindowBuilder::new()
        .with_title(t.settings_title)
        .with_inner_size(tao::dpi::LogicalSize::new(420.0, 280.0))
        .with_resizable(false)
        .build(event_loop)
        .expect("create settings window");

    let proxy = proxy.clone();
    let html = settings_html(lang);
    let webview = WebViewBuilder::new()
        .with_html(&html)
        .with_ipc_handler(move |req| {
            let body = req.body().to_string();
            let _ = proxy.send_event(UserEvent::PrefsMessage(body));
        })
        .build(&window)
        .expect("create settings webview");

    *slot.borrow_mut() = Some(SettingsWindow { window, webview });
}

#[cfg(target_os = "macos")]
fn refresh_settings_ui(slot: &RefCell<Option<SettingsWindow>>, lang: Lang) {
    let guard = slot.borrow();
    let Some(settings) = guard.as_ref() else {
        return;
    };
    let t = lang.strings();
    settings.window.set_title(t.settings_title);
    let _ = settings.webview.load_html(&settings_html(lang));
}

fn origin_of(raw: &str) -> String {
    Url::parse(raw)
        .ok()
        .and_then(|u| {
            u.host_str().map(|h| {
                format!(
                    "{}://{}:{}",
                    u.scheme(),
                    h,
                    u.port_or_known_default().unwrap_or(0)
                )
            })
        })
        .unwrap_or_else(|| "<invalid url>".to_string())
}

fn apply_language(
    next: Lang,
    lang_cell: &Cell<Lang>,
    #[cfg(target_os = "macos")] menu_slot: &RefCell<Option<muda::Menu>>,
    #[cfg(target_os = "macos")] settings_slot: &RefCell<Option<SettingsWindow>>,
) {
    if next == lang_cell.get() {
        return;
    }
    lang_cell.set(next);
    if let Err(e) = i18n::save_preference(next) {
        eprintln!("kimini: failed to save language preference: {e}");
    }
    eprintln!("kimini: lang={}", next.code());

    #[cfg(target_os = "macos")]
    {
        *menu_slot.borrow_mut() = None;
        *menu_slot.borrow_mut() = Some(install_menu(&next.strings()));
        refresh_settings_ui(settings_slot, next);
    }
}

fn main() -> wry::Result<()> {
    let lang = Lang::resolve();
    let strings = lang.strings();
    eprintln!("kimini: lang={}", lang.code());

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    #[cfg(target_os = "macos")]
    let menu_slot: Rc<RefCell<Option<muda::Menu>>> = Rc::new(RefCell::new(None));
    #[cfg(target_os = "macos")]
    {
        *menu_slot.borrow_mut() = Some(install_menu(&strings));
        let menu_proxy = proxy.clone();
        std::thread::spawn(move || {
            while let Ok(event) = muda::MenuEvent::receiver().recv() {
                let _ = menu_proxy.send_event(UserEvent::Menu(event.id));
            }
        });
    }

    let lang_cell = Rc::new(Cell::new(lang));

    #[cfg(target_os = "macos")]
    let settings_slot: Rc<RefCell<Option<SettingsWindow>>> = Rc::new(RefCell::new(None));

    let window = WindowBuilder::new()
        .with_title(APP_TITLE)
        .with_inner_size(tao::dpi::LogicalSize::new(1440.0, 900.0))
        .build(&event_loop)
        .expect("create window");
    let main_id: WindowId = window.id();

    let explicit = explicit_url();
    match &explicit {
        Some(url) => eprintln!("kimini: loading {}", origin_of(url)),
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
                eprintln!("kimini: loading {}", origin_of(&url));
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
                    open_or_focus_settings(target, &proxy, lang_cell.get(), &settings_slot);
                }
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
