# Native GUI Framework Selection

Date: 2026-07-18

## Decision

For Kimini's stated product, use **GPUI + gpui-component for the primary GUI**, subject to a short G0 proving long-chat virtualization, Chinese/Japanese IME, VoiceOver, and a child browser surface. Keep the **browser as a separate module** with its own runtime and lifecycle.

This choice follows the product constraint: the permanent chat, coding, diff, terminal, approval, file, and settings surfaces should be Rust-rendered rather than DOM-rendered. Tauri v2 and Electron remain valid products with different priorities:

- Choose **Tauri v2** when rapid cross-platform delivery and a small installer matter more than a native-rendered primary UI.
- Choose **Electron** when one bundled Chromium and the shortest route to deeply integrated browser automation matter more than idle resources and a Rust-native UI.
- Choose **GPUI** when the native UI requirement, low browser-off baseline, and coding-workload control are primary.

Rust Edition 2024 does not constrain the selection. GPUI and Tauri are Rust crates usable from an Edition 2024 application. Electron would keep Rust behind process IPC or a native Node module rather than use Rust as the renderer.

## G0 implementation update

The framework choice has now crossed the first implementation threshold:

- The native daemon/session/streaming path, variable-height conversation, composer, interactions, and bounded WRY child surface compile and run against the real local Kimi daemon.
- crates.io GPUI 0.2.2 predates Zed's May 2026 AccessKit integration. Kimini therefore tracks the current Zed and gpui-component Git sources through exact `Cargo.lock` revisions; the current macOS accessibility tree exposes the application, headings, status, session list, buttons, conversation articles, and text area.
- WRY child-view bounds, focus hand-off, explicit open, and explicit close work without making WebKit a permanent renderer. Opening creates WebContent, GPU, and Networking helpers. Closing removes WebContent; macOS retains pooled GPU and Networking helpers until host exit. A strict zero-residue lifecycle therefore requires the isolated browser companion process.
- The choice remains conditional on the outstanding packaged CJK, 1,000-turn, overlap, frame-time, and matched process-family measurements. These are product gates, not assumptions inherited from Zed.

## Direct comparison for Kimini

| Criterion | GPUI + gpui-component | Tauri v2 | Electron |
|---|---|---|---|
| **Rendering architecture** | Rust view/element trees are laid out and rendered directly through GPU backends. GPUI is hybrid retained/immediate mode; macOS uses Metal. No browser engine is required for the primary UI. | A Rust core owns windows and privileged state; system WebView processes render HTML/CSS/JavaScript. Current desktop engines are WKWebView on macOS, WebView2 on Windows, and WebKitGTK on Linux. | Bundles Chromium and Node.js. The main process owns windows and each window or web embed receives Chromium renderer infrastructure. |
| **Fit for the primary Kimini GUI** | **Best fit.** Dense code, selectable text, custom layout, virtual lists, editors, and keyboard-heavy interaction match GPUI's Zed workload. gpui-component reduces the widget work. | **Weak fit for this brief.** It would replace the existing WRY shell with a more complete framework while retaining a WebView-rendered primary UI. It is a strong fit only if that constraint is relaxed. | **Weakest fit for this brief.** It can reproduce a Codex-style web UI quickly, while making Chromium the permanent application renderer. |
| **Embedded human browser** | No upstream built-in browser. GPUI exposes native window handles, so application code can attach an on-demand WRY child WebView; focus, clipping, z-order, resize, and accessibility interop remain product work. | Strong. Multiple WebViews and WebView windows are native framework concepts. Engine behavior still varies by operating system. | Strongest integrated option. `WebContentsView` embeds separately controlled web content inside an Electron `BaseWindow`. |
| **Model-controlled browser** | Strong after adding a separate browser broker. Use isolated Chromium with Playwright/CDP for consistent automation; the visible surface can begin with bounded frames and later move to a child/off-screen Chromium surface. | A system WebView alone does not provide one consistent cross-platform automation contract. A separate Chromium/Playwright broker is still the dependable path for agent use. | Strongest shortest path. Electron exposes Chromium web contents and a CDP-compatible `debugger` transport in the same runtime. |
| **External Chrome / Computer Use** | Framework-neutral. Implement external Chrome through an extension or CDP adapter, and Computer Use through a separately consented OS helper. | Same framework-neutral adapters. Tauri plugins may help with OS integration but do not remove the trust boundary. | Same adapters; Node/Chromium integration reduces plumbing. Permissions and process isolation remain required. |
| **Browser-off resource baseline** | **Best architectural ceiling.** The primary GUI can run without WebKit/Chromium/JavaScript renderer processes. Actual cost depends on text layout, invalidation, caching, and the Kimi daemon. | **Middle.** It avoids bundling Chromium, while a system WebView remains active because the primary UI lives in it. Small distribution size does not guarantee low runtime memory. | **Largest fixed runtime of the three.** Chromium, Node.js, and the multi-process model ship with the app. Workload measurements are still required before making numeric claims. |
| **Browser-on resource baseline** | Adds WebView or Chromium work on demand. After an embedded WKWebView closes, macOS may retain pooled GPU/Networking helpers; an isolated broker is required for complete teardown. A full agent browser costs according to its engine and tabs. | Already pays for the primary system WebView, then may add more WebViews or a Chromium automation worker. | Chromium is already resident; additional web contents and browser targets add renderer and utility work. This is convenient, not free. |
| **macOS / Windows / Linux** | Zed ships a stable Windows build, proving that the Windows backend is real. GPUI's standalone getting-started guide still explicitly asks third-party users to be on macOS or Linux, so Windows is not yet a clean standalone support promise. Treat independent-app parity as a product gate. | Mature desktop coverage and tooling on all three. The web engine changes across platforms, so rendering and browser behavior can diverge. | Mature desktop coverage on all three with a bundled Chromium version, giving the most consistent web rendering across operating systems. |
| **Visual polish** | Highest custom-native control and a directly relevant reference product in Zed. Achieving polish requires Rust component work; gpui-component supplies themes, Markdown, code highlighting, docks, editors, and virtual lists. | CSS offers the fastest styling iteration and a large component ecosystem. Native-looking behavior requires deliberate design and platform work. | Same CSS advantage plus the largest desktop-web ecosystem. Codex demonstrates the attainable result; Electron itself does not supply the design quality. |
| **Accessibility and IME** | GPUI integrates AccessKit and exposes marked-text/input-handler APIs. Zed proves serious editor use, while Kimini must still prove CJK composition, candidate-window placement, VoiceOver semantics, and live streaming announcements in its exact components. | Browser-grade IME and semantic HTML accessibility are available through each system WebView. Correct markup remains application work, and engine/platform differences need coverage. | Strongest and most uniform baseline through Chromium's IME and accessibility tree. Correct semantics and focus behavior remain application work. |
| **Rust integration** | **Direct.** UI state, protocol projection, commands, and rendering stay in Rust with no renderer IPC boundary. | **Good backend, split frontend.** Rust owns privileged logic; UI state crosses WebView IPC to JavaScript or Rust/Wasm frontend code. | **Indirect.** The renderer and main process are JavaScript/TypeScript; the Kimi Rust layer is normally a child app-server/sidecar or native module. |
| **Maturity** | Highest risk. GPUI explicitly remains pre-1.0 with frequent breaking changes and comparatively sparse standalone documentation. Pin exact revisions. | Stable v2 ecosystem with documented security capabilities, plugins, IPC, bundling, signing, and updating. | Most mature desktop-web ecosystem, with regular Chromium/Node/Electron upgrade and security work. |
| **Packaging and updates** | GPUI is a UI framework; application bundling, signing, updating, and per-platform release work remain Kimini responsibilities. The existing Kimini scripts already cover macOS. | Best integrated Rust-oriented story: the CLI builds platform bundles/installers and supports signing, stores, and updater plugins. | Mature through Electron Forge and related tools. The distributable includes the Electron binary and is structurally larger. |
| **Implementation effort** | Highest initial UI effort. The expensive coding primitives are available, yet integration and product components remain Rust work. | Lowest if reusing Kimi Code Web or a web frontend; medium if rebuilding a highly optimized bespoke frontend. | Lowest for a Codex-like web implementation and browser embedding; adds Node/Electron build, IPC, sandbox, and update surfaces. |
| **Primary risk** | Framework churn plus unproven independent-app IME/accessibility/browser-child-surface behavior. | Kimini spends migration effort and still retains the WebView architecture it set out to leave; system-engine differences complicate deterministic browser automation. | Permanent Chromium cost, larger artifacts/process tree, and two application stacks when the agent core remains Rust. |
| **Recommendation** | **Winner for the primary GUI, conditional on G0.** | **Fallback among these three if native rendering is negotiable.** | **Choose only if browser integration and delivery speed become dominant product goals.** |

## What Zed currently solves about browser access

This distinction matters: Zed can **access the web and drive an external browser**, while upstream Zed still does not display a general web page inside a GPUI pane.

Status at upstream commit [`9d7ab044`](https://github.com/zed-industries/zed/commit/9d7ab044366fb266cecb30b214aea8b7b94c032d) (2026-07-17):

| Capability | Current upstream status | Evidence |
|---|---|---|
| Agent reads/searches the web | Shipped. `fetch` performs HTTP requests and converts HTML to Markdown; `search_web` returns links and snippets. | [Agent tools](https://zed.dev/docs/ai/tools#fetch), [`fetch_tool.rs`](https://github.com/zed-industries/zed/blob/9d7ab044366fb266cecb30b214aea8b7b94c032d/crates/agent/src/tools/fetch_tool.rs), [`web_search_tool.rs`](https://github.com/zed-industries/zed/blob/9d7ab044366fb266cecb30b214aea8b7b94c032d/crates/agent/src/tools/web_search_tool.rs) |
| Agent controls a browser | Available through an external MCP server; Zed's official MCP guide lists Puppeteer. This gives the model browser tools, not an in-pane browser surface. | [Zed MCP guide](https://zed.dev/docs/ai/mcp#installing-mcp-servers) |
| Zed displays normal web pages in-app | Not shipped. HTTP(S) links route to GPUI's `open_url`, which invokes the system browser. Upstream has no WRY dependency or WebView element. | [`Workspace::open_url_or_file`](https://github.com/zed-industries/zed/blob/9d7ab044366fb266cecb30b214aea8b7b94c032d/crates/workspace/src/workspace.rs#L4715-L4763), [`App::open_url`](https://github.com/zed-industries/zed/blob/9d7ab044366fb266cecb30b214aea8b7b94c032d/crates/gpui/src/app.rs#L1406-L1410) |
| Extensions create a custom WebView/tab/panel | Not supported. The documented extension surfaces are languages, debuggers, themes, icon themes, snippets, and MCP servers; the WebView umbrella request remains open. | [Extension features](https://zed.dev/docs/extensions/developing-extensions#extension-features), [issue #21208](https://github.com/zed-industries/zed/issues/21208) |
| GPUI application code attaches a native child view | Shipped as low-level groundwork. GPUI `Window` implements `HasWindowHandle` and `HasDisplayHandle`, which is enough for application code to call WRY `build_as_child`. It is not a reusable GPUI WebView component. | [merged PR #24327 / commit `1c494526`](https://github.com/zed-industries/zed/commit/1c494526d8ec060151a8e7e5022904daf1effbbc), [current `Window` implementation](https://github.com/zed-industries/zed/blob/9d7ab044366fb266cecb30b214aea8b7b94c032d/crates/gpui/src/window.rs#L6138-L6150), [merged fix #24545 / commit `897e172c`](https://github.com/zed-industries/zed/commit/897e172cb7b34fb4b4d12b9446a19e5d23455725) |

The `gpui_web` crate is also easy to misread: it is a WASM platform backend for running GPUI **inside a browser page**, not an API for embedding a browser inside a desktop GPUI window ([source](https://github.com/zed-industries/zed/blob/9d7ab044366fb266cecb30b214aea8b7b94c032d/crates/gpui_web/Cargo.toml#L15-L49)).

### What extensions can solve today

There are three different extension outcomes, and only the first two exist today:

1. **Agent-controlled external browser:** Zed's official extension registry contains [Chrome DevTools MCP](https://zed.dev/extensions/chrome-devtools-mcp), [Playwright MCP](https://zed.dev/extensions/mcp-server-playwright), and [BrowserTools](https://zed.dev/extensions/browser-tools-context-server). They expose navigation, interaction, page state, screenshots, console/network data, and audits to the Agent Panel while Chrome or Chromium remains a separate process. This is the likely source of the impression that Zed has browser access.
2. **Extension-launched companion window:** the community [Excalidraw extension](https://github.com/arindampradhan/excalidraw-zed-extension) uses Zed's `process:exec` capability to launch a Rust `wry + tao` preview binary. A native WebView window opens beside Zed and updates independently. This proves the companion-process pattern, not an editor-pane WebView.
3. **Extension-owned in-pane WebView:** unavailable in the current WASM extension API. Issue [#21208](https://github.com/zed-industries/zed/issues/21208) remains the upstream request for custom WebViews and workspace items.

Kimini owns its host application, so it does not need to wait for Zed's third-party extension ABI. The minimal first implementation should launch one visible, isolated Chromium through Playwright MCP or Chrome DevTools MCP and expose its state and user-takeover controls in GPUI. Add a general plugin API only when a second provider creates a real abstraction requirement. The companion-process boundary also keeps the browser fully unloadable when unused.

### What the rejected upstream prototypes established

- [PR #52447](https://github.com/zed-industries/zed/pull/52447), head [`a0d33c88`](https://github.com/zed-industries/zed/commit/a0d33c888ae00cd6fd56fe20fa31989159eb6169), implemented a WRY browser tab. It worked as an OS child view on the demonstrated X11 path and was intended for macOS/Windows; Wayland fell back to the system browser. Its author reported jitter and GPUI dropdowns appearing behind the page. Zed closed it on 2026-04-02 because the design needed more discussion.
- [PR #54433](https://github.com/zed-industries/zed/pull/54433), head [`759b24ff`](https://github.com/zed-industries/zed/commit/759b24ffb13965bd9dd24f8e7bbfb2d642931498), implemented a feature-gated GPUI `WebView` element: `WKWebView` as an `NSView` child on macOS, a child `HWND`/WebView2 on Windows, and an X11 child bridged through GDK on a dedicated GTK thread. Wayland could only open a separate GTK window. Zed closed it on 2026-04-23 because the dependency/binary cost was too large for the immediate notebook use case and the Linux path remained incomplete.
- [PR #48157](https://github.com/zed-industries/zed/pull/48157), head [`1fb0d7d5`](https://github.com/zed-industries/zed/commit/1fb0d7d575a762787b03178f7c64fa495053bdfc), used WRY for HTML notebook output. The prototype reported that GPUI keybindings could not copy WebView text and that the native WebView overlaid GPUI elements.

These prototypes synchronize rectangular layout bounds to the native child view ([element source](https://github.com/zed-industries/zed/blob/759b24ffb13965bd9dd24f8e7bbfb2d642931498/crates/gpui/src/elements/webview.rs#L788-L877)). They do not integrate GPUI clip masks, rounded clipping, or scene z-order. Consequently, an OS WebView stays above GPUI popovers, menus, tooltips, selections, and other overlapping elements; scroll clipping and focus hand-off require explicit host logic. A constrained browser pane can work by reserving a rectangular region, keeping controls outside that region, and hiding the child view before showing overlapping GPUI surfaces. Linux Wayland still needs a separate window, frame streaming/off-screen compositing, or another browser backend.

For Kimini this strengthens the **macOS-first GPUI + WRY** option: the necessary raw-window-handle escape hatch is already upstream, so no GPUI fork is required for the first child-surface spike. This is an application-level integration point, not a Zed WASM-extension capability. It does not remove that spike from G0. Human preview/OAuth can use the constrained system WebView; deterministic model browsing should remain an isolated Chromium/Playwright/CDP capability, with its visible output either hosted in the same bounded region or composited as frames.

## Main GUI and browser module are separate decisions

The primary GUI framework should optimize the always-running coding experience. The browser module should optimize web compatibility, automation, isolation, and user takeover.

```text
Kimini Native
├── GPUI primary GUI
│   ├── sessions, chat, Markdown, code, diff, terminal
│   └── approvals, files, settings, native app chrome
└── on-demand Browser Broker
    ├── human preview / OAuth -> existing WRY system child WebView
    ├── agent Browser Use      -> isolated Chromium + Playwright/CDP
    ├── external Chrome        -> extension or CDP adapter
    └── Computer Use           -> separate OS-control helper and consent
```

Tauri is unnecessary for the browser module alone because Kimini already has WRY, Tauri's underlying WebView abstraction. Electron's `WebContentsView` belongs to Electron windows and is not a reusable child widget for a GPUI window. If the GPUI child-surface spike fails, use WRY for human browsing and an isolated Chromium frame stream for agent browsing before considering a wholesale GUI-framework change.

Codex supports this separation as a product lesson. The inspected local build uses Electron/Chromium for its shell and carries strong `iab`, external-browser, CDP, and Computer Use capabilities, while its Rust app-server remains a separate process. That is evidence for Codex's priorities and protocol boundary; it is not a performance baseline for Kimini. The same snapshot had a large bundle and a multi-process tree, so Kimini should measure native browser-off and browser-on modes separately.

## Decision tree

```text
Must the permanent chat/coding UI be Rust-rendered?
├── Yes
│   └── Start with GPUI G0
│       ├── Passes long-chat, IME, VoiceOver, child-surface, Windows gates -> GPUI
│       └── Fails a fundamental gate
│           ├── WebView primary UI is acceptable -> Tauri v2
│           └── WebView primary UI remains unacceptable -> evaluate another native toolkit
└── No
    ├── Lowest fixed runtime and small distribution matter most -> Tauri v2
    └── Consistent bundled Chromium and deepest browser integration matter most -> Electron

Browser requirement, independently:
├── Human preview / OAuth -> WRY system WebView
├── Deterministic agent browsing -> isolated Chromium + Playwright/CDP
├── User's existing Chrome profile -> extension/CDP adapter with separate consent
└── Whole desktop control -> separate Computer Use capability
```

## G0 acceptance gates

Build only daemon connection, session list, prompt input, streamed Markdown/code, and a variable-height virtual conversation. Add one disposable WRY child-surface spike. Stop before terminal, settings, browser automation, or a custom design system.

Continue with GPUI only if the prototype demonstrates:

- correct Chinese and Japanese IME composition, candidate windows, cursor placement, selection, clipboard, emoji, and dead keys;
- useful VoiceOver roles, names, focus order, keyboard-only operation, and streaming-content announcements;
- stable anchoring and scrolling through at least 1,000 mixed-height turns during streaming;
- selectable Markdown, long code blocks, syntax highlighting, tool cards, and large diffs without rebuilding the entire conversation per token;
- correct browser child-surface focus, clipping, resize, z-order, model/user takeover, and cleanup;
- a clear same-workload improvement over the current WRY client in browser-off idle cost, input latency, frame pacing, long-session loading, and streaming.

No numeric performance winner should be published before this same-workload run. Measure the host, Kimi daemon, and all WebKit/Chromium/GPU/network helpers as one product process tree.

## Primary sources

- GPUI: [official architecture, maturity, renderers, and platform backends](https://github.com/zed-industries/zed/blob/main/crates/gpui/README.md), [official accessibility implementation guide](https://github.com/zed-industries/zed/blob/main/crates/gpui/src/_accessibility.rs), [official input-handler source](https://github.com/zed-industries/zed/blob/main/crates/gpui/src/input.rs), [Zed's stable Windows distribution](https://zed.dev/docs/windows)
- gpui-component: [official repository and component inventory](https://github.com/longbridge/gpui-component)
- Tauri v2: [official architecture](https://v2.tauri.app/concept/architecture/), [official process and WebView model](https://v2.tauri.app/concept/process-model/), [official platform WebView versions](https://v2.tauri.app/reference/webview-versions/), [official distribution tooling](https://v2.tauri.app/distribute/)
- Electron: [official introduction](https://www.electronjs.org/docs/latest/), [official process model](https://www.electronjs.org/docs/latest/tutorial/process-model), [official web embeds](https://www.electronjs.org/docs/latest/tutorial/web-embeds), [official CDP debugger transport](https://www.electronjs.org/docs/latest/api/debugger), [official accessibility model](https://www.electronjs.org/docs/latest/tutorial/accessibility), [official packaging guide](https://www.electronjs.org/docs/latest/tutorial/tutorial-packaging)
- Browser module: [WRY child WebViews](https://docs.rs/wry/latest/wry/struct.WebViewBuilder.html#method.build_as_child), [Playwright MCP](https://github.com/microsoft/playwright-mcp), [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/), [CEF off-screen rendering](https://chromiumembedded.github.io/cef/general_usage.html#off-screen-rendering)
- Local Codex evidence: [`codex-reference-architecture.md`](./codex-reference-architecture.md)
