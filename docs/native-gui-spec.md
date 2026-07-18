# Native Kimi Code GUI Specification

Status: G0 implementation checkpoint; final G0 acceptance remains open

Date: 2026-07-18

Product: Kimini
Upstream baseline: Kimi Code `3086e4703992fbbe7a41379405ee243713ad9ced`

This is the authoritative implementation specification. The framework and Codex research notes remain supporting evidence; when they conflict with this document, this document wins.

## Problem Statement

Kimi Code Web exposes the complete Kimi Code workflow, but its permanent HTML, CSS, JavaScript, DOM, WebKit, and GPU process tree carries substantial long-session memory and repaint cost. The current Kimini application makes that web experience easier to launch, yet it remains a thin WebView shell and inherits the same rendering behavior.

Users need a real desktop agent workstation with native responsiveness, predictable long-session behavior, first-class keyboard and accessibility support, and the complete Kimi Code interaction model. Kimi Code Web must remain available during the transition and as a compatibility fallback. Future browser automation must be able to run visibly and, later, inside Kimini without turning the permanent application shell into a browser renderer.

## Solution

Build a Rust Edition 2024 desktop application whose permanent interface is rendered with GPUI and gpui-component. Kimini connects directly to the existing Kimi daemon over its public `/api/v1` REST and WebSocket contracts, projects wire events into a deterministic application model, and renders workspaces, sessions, conversations, tools, approvals, questions, files, diffs, terminals, tasks, settings, and authentication natively.

Ship two independent applications. **Kimini** is the GPUI-native client and **Kimini Web** preserves the current WRY compatibility surface. They have separate binaries, bundle identifiers, windows, and release artifacts, while the daemon remains the source of truth so both can operate against the same sessions without migration.

Browser functionality is an on-demand subsystem. The first browser delivery launches an isolated visible Chromium controlled through Playwright/CDP and exposed to Kimi through MCP. Later delivery adds an in-app browser view while preserving the low browser-off baseline. Human browsing, agent Browser Use, external Chrome, full CDP, and whole-desktop control remain separate permissions.

## User Stories

### Launch and connection

1. As a Kimi Code user, I want Kimini to discover my local daemon automatically, so that the native app starts without configuration.
2. As a Kimi Code user, I want Kimini to start the daemon when it is absent, so that I do not need a terminal before opening the app.
3. As an advanced user, I want to provide an explicit server origin, so that I can connect to a non-default local instance.
4. As a user, I want clear connecting, reconnecting, incompatible-server, and offline states, so that transport problems are understandable.
5. As a user, I want a one-click route to Kimi Code Web when native mode cannot support the connected server, so that I can continue working.

### Workspaces and sessions

6. As a developer, I want to add, rename, reorder, collapse, and remove workspaces, so that sessions follow my project organization.
7. As a developer, I want to browse for a workspace folder with a native picker, so that choosing a project is fast and safe.
8. As a developer, I want to create a session in a selected workspace, so that its working directory is correct.
9. As a developer, I want paginated session lists grouped by workspace, so that a large history remains responsive.
10. As a developer, I want to search sessions from the keyboard, so that I can jump to older work quickly.
11. As a developer, I want active, approval-required, question-required, completed, cancelled, and failed indicators in the session list, so that I can monitor several agents at once.
12. As a developer, I want to rename, archive, restore, fork, undo, compact, and export sessions, so that native mode preserves Kimi Code lifecycle operations.
13. As a developer, I want deep links to open a session outside the first page, so that shared links remain useful.
14. As a developer, I want child sessions and BTW side chats associated with their parent, so that focused conversations do not pollute the main list.
15. As a developer, I want per-session drafts and pane state restored locally, so that switching sessions does not lose context.

### Conversation and composer

16. As a user, I want recent history to appear immediately from an atomic snapshot, so that a session opens in a consistent state.
17. As a user, I want older messages loaded on demand while preserving scroll position, so that long histories remain usable.
18. As a user, I want assistant text and thinking to stream smoothly, so that progress is visible without per-token layout churn.
19. As a user, I want user, assistant, system, compaction, cron, tool, image, video, and file content represented faithfully, so that no meaningful transcript data disappears.
20. As a user, I want native Markdown, tables, lists, links, code blocks, syntax highlighting, and selectable text, so that responses remain readable and copyable.
21. As a user, I want Mermaid and mathematical content rendered on demand with accessible source fallbacks, so that rich responses retain their meaning.
22. As a user, I want tool calls grouped with status, arguments, bounded live output, results, media, and diffs, so that agent work is understandable without overwhelming the chat.
23. As a user, I want the viewport anchored while streaming unless I intentionally scroll away, so that reading is stable.
24. As a user, I want text, images, videos, and files added by picker, paste, or drag and drop, so that multimodal prompts match Kimi Code Web.
25. As a keyboard user, I want Enter to send, Shift+Enter to add a line, and Cmd/Ctrl+Enter to send from expanded mode, so that composing is predictable.
26. As a keyboard user, I want Cmd/Ctrl+S to steer queued text into the running turn, so that native mode matches Kimi Code behavior.
27. As a keyboard user, I want slash commands, file mentions, prompt history, and arrow-key navigation, so that frequent actions stay fast.
28. As a user, I want a separate stop action while new prompts remain queueable, so that sending and interrupting cannot be confused.

### Agent interaction and controls

29. As a user, I want model, thinking effort, permission mode, plan mode, swarm mode, and goal controls in the composer, so that each turn uses the intended runtime configuration.
30. As a user, I want live context usage and current model from session runtime state, so that stale profile metadata cannot mislead me.
31. As a user, I want approvals displayed inline with command, file, URL, search, invocation, and plan context, so that decisions are informed.
32. As a keyboard user, I want visible numeric shortcuts for approval choices, so that repetitive decisions are quick.
33. As a user, I want approval decisions to support one-time approval, session approval, rejection, and feedback where the daemon offers them, so that native behavior preserves policy semantics.
34. As a user, I want single-select, multi-select, free-form, multi-step, submit, and dismiss question flows, so that agent clarification works fully.
35. As a user, I want pending approvals and questions restored after reconnect, so that prompts cannot become stranded behind invisible interaction state.
36. As a user, I want goals to show active, paused, completed, and blocked state with their controls, so that long-running work stays legible.
37. As a user, I want task and subagent rosters with live progress, final status, bounded output, and cancellation, so that parallel work is observable.
38. As a user, I want side-chat text and thinking isolated from the parent transcript, so that BTW work remains focused.

### Coding surfaces

39. As a developer, I want a native workspace tree with lazy directories and git status, so that project context is available beside the conversation.
40. As a developer, I want file-name search and content search, so that I can navigate without leaving Kimini.
41. As a developer, I want read-only text, image, audio, video, and binary metadata previews, so that tool references can be inspected safely.
42. As a developer, I want syntax-highlighted file previews with line numbers and direct line navigation, so that code references are useful.
43. As a developer, I want unified diffs with additions, deletions, context, and file summaries, so that agent changes are reviewable.
44. As a developer, I want repository branch, ahead/behind, pull request, and change totals, so that workspace status is visible.
45. As a developer, I want to open or reveal a file in a configured external application, so that editing can remain in my preferred IDE.
46. As a developer, I want native terminal tabs backed by Kimi daemon terminals, so that shell work shares the session working directory and lifecycle.
47. As a terminal user, I want resize, input, scrollback, reconnect backfill, exit status, and close behavior, so that terminal sessions survive ordinary UI changes.
48. As a user, I want skills listed and activated for a session or workspace, so that Kimi capabilities are discoverable from the GUI.

### Settings, platform behavior, and accessibility

49. As a user, I want managed login, provider configuration, model refresh, logout, and authentication status, so that setup does not require editing config files.
50. As a user, I want English and Simplified Chinese UI, so that the application follows my language preference.
51. As a user, I want light, dark, and system appearance with stable design tokens, so that Kimini feels coherent across every surface.
52. As a keyboard user, I want complete focus order, focus restoration, menu shortcuts, command search, and dialog trapping, so that a pointer is optional.
53. As a screen-reader user, I want semantic roles, names, states, live announcements, and useful reading order, so that agent activity is accessible.
54. As a CJK user, I want correct marked text, candidate placement, composition, selection, dead keys, and emoji input, so that native text entry is dependable.
55. As a motion-sensitive user, I want reduced motion honored and nonessential animation disabled, so that the interface remains comfortable.
56. As a user, I want optional completion, approval, and question notifications without leaking question text by default, so that background sessions can ask for attention safely.

### Browser capability

57. As a user, I want browser processes absent until I request browser work, so that ordinary coding retains the native resource baseline.
58. As an agent user, I want Kimi to navigate, inspect, click, type, capture, and evaluate pages through a browser tool server, so that web tasks can complete autonomously.
59. As a user, I want the controlled browser to be visible and clearly marked as agent-controlled, so that automation is observable.
60. As a user, I want an explicit user-takeover state, so that agent input pauses while I interact with the page.
61. As a user, I want the browser view available in a companion window first and in a bounded Kimini pane later, so that delivery can improve without replacing the native shell.
62. As a user, I want browser profile, downloads, clipboard, camera, microphone, location, popups, and credentials governed by explicit permissions, so that browser automation has clear trust boundaries.
63. As a user, I want authentication values entered through host-owned UI and hidden from model output, screenshots, logs, and transcript history, so that secrets remain private.
64. As a user, I want external Chrome, embedded browsing, full CDP, and Computer Use enabled independently, so that granting one capability does not silently grant another.
65. As a user, I want closing the browser to terminate its renderer and helper processes, so that resource use returns near the browser-off baseline.

### Reliability and diagnostics

66. As a user, I want sleep, daemon restart, socket loss, journal rollover, and epoch changes recovered automatically, so that sessions do not need a manual reload.
67. As a user, I want errors attached to the relevant session and operation, so that one failed agent does not look like an application-wide failure.
68. As a maintainer, I want redacted client diagnostics included in session exports, so that failures can be investigated without exposing tokens or credentials.
69. As a maintainer, I want unknown protocol fields tolerated and unsupported required capabilities surfaced clearly, so that upstream evolution fails safely.
70. As a maintainer, I want browser-off and browser-on process trees measured separately, so that performance claims reflect the whole product.

## Implementation Decisions

### Product and migration boundary

- Kimini is one Rust Edition 2024 package with two binaries and two macOS bundles: `kimini` / `app.kimini` for native GPUI, and `kimini-web` / `app.kimini.web` for the WRY compatibility app.
- The applications can be installed and run simultaneously. Native is the primary development target; Kimini Web remains an independently launchable fallback throughout the parity work.
- Both modes use the same daemon discovery and bearer-token rules. No session database or transcript migration is introduced.
- Native parity means user-visible behavior and daemon semantics match Kimi Code Web. DOM structure and web-only implementation details are not parity requirements.
- Kimi Code Web's current design system is the visual baseline: 264-point sidebar, 760-point normal conversation width, 920-point wide content, 4-point spacing scale, restrained radii and shadows, native light/dark/system appearance, and no decorative animation that creates idle repaint.

### Process and ownership model

```text
Kimi daemon
  ⇅ REST snapshots/commands + journaled WebSocket events
Kimini transport runtime
  → wire decoding → protocol projection → bounded AppInput queue
Kimini GPUI main thread
  → AppModel reducer → view model → native scene
  → user intent → AppEffect queue → transport runtime

On demand only:
Browser Broker ⇄ MCP tools for Kimi
               ⇄ CDP/Playwright for isolated Chromium
               ⇄ authenticated local IPC for Kimini browser state and frames
```

- GPUI owns windows, focus, menus, accessibility, and rendering on the main thread.
- The implemented G0 uses GPUI background tasks for blocking local REST calls and one dedicated WebSocket thread. A bounded `async-channel` transfers lossless events to the main thread without polling or empty redraws.
- A future Browser Broker owns its own async runtime and process lifecycle. G0 does not add Tokio to the permanent native client.
- Communication uses bounded channels. The UI never blocks on network or process I/O.
- The daemon owns durable sessions, messages, approvals, questions, tasks, terminals, configuration, and authentication. Native caches are disposable projections.

### Module boundaries

- `daemon` retains discovery, health probing, CLI resolution, token loading, and daemon startup.
- `protocol` owns wire DTOs, WebSocket control frames, event envelopes, snapshots, sessions, messages, approvals, and questions.
- `model` owns the pure application reducer, sequence/epoch cursor rules, conversation state, streaming tails, and derived session state.
- `api` owns authenticated REST commands and the bounded WebSocket worker. Bearer handling does not leak into views.
- `native` owns GPUI bootstrap, commands, lifecycle, presentation cache, views, theme, input, accessibility, and the bounded browser child surface.
- `legacy_web` contains the current WRY shell and remains isolated from native state.
- `native/browser` contains only human-preview address policy, WRY lifecycle, and the GPUI child-surface bridge. The future agent Browser Broker remains a separate subsystem; a general provider interface waits until a second backend needs it.

### Protocol contract

- `/api/v1` remains the client boundary across Kimi backend generations. Kimini does not import Kimi agent-core packages or parse CLI/TUI output.
- REST wire definitions follow the daemon OpenAPI document; WebSocket control/event definitions follow its AsyncAPI document and the proven Kimi Code Web projection semantics.
- Startup reads health and metadata, records server version/backend/capabilities, then loads workspaces and sessions.
- Opening a session always follows `snapshot → seed in-flight state → subscribe at {seq, epoch}`. Live events older than the accepted watermark are ignored.
- `resync_required`, an epoch change, a cursor gap, or a stale half-open socket re-enters the snapshot flow. Existing older pages are retained only where they precede the new snapshot window without overlap.
- Authoritative snapshot and completed-event state replaces optimistic state. Optimistic user messages are reconciled by prompt/message identity before content heuristics.
- Pending approvals and questions live in `AppModel`, survive view recreation, and clear only on an authoritative resolution, expiration, dismissal, or snapshot.
- Unknown optional fields and event variants are preserved for diagnostics and otherwise ignored. A missing required capability produces an incompatible-server state with an Open Web action.
- REST and WebSocket requests carry stable client identity/version/UI-mode metadata. Bearer tokens remain in memory, never enter URLs, logs, telemetry, exports, or rendered state.

### State, concurrency, and backpressure

- `AppModel` is a deterministic reducer of `AppInput` into state plus `AppEffect`. Inputs include projected daemon events, command results, timers, window events, and user intents.
- Lossless events include snapshots, lifecycle transitions, text/thinking deltas, approvals, questions, and authoritative completions. Adjacent compatible deltas may be coalesced without dropping bytes.
- Best-effort streams include repetitive progress and terminal/tool output already recoverable from the daemon. They use byte caps, truncation markers, and backfill rather than unbounded queues.
- UI invalidation is batched to at most one update per display frame. Streaming never rebuilds the complete conversation tree.
- Conversation history uses variable-height virtualization, stable item identities, scroll anchoring, and compact off-screen models.
- Markdown, highlight, glyph, image, diff, terminal scrollback, and browser-frame caches have explicit byte budgets and LRU eviction. Background sessions retain domain state but release expensive render state.

### Native interface design

```text
┌──────────────────────────────────────────────────────────────────────┐
│ native title/toolbar: workspace · session · status · global actions │
├──────────────┬───────────────────────────────┬───────────────────────┤
│ workspaces   │ conversation header           │ one detail surface    │
│ sessions     │ virtual transcript            │ file / diff / task    │
│ search       │ approvals / questions         │ thinking / side chat  │
│ new/settings │ composer + runtime controls   │ terminal / browser    │
└──────────────┴───────────────────────────────┴───────────────────────┘
```

- Desktop layout has one collapsible/resizable left sidebar, one central conversation, and one shared resizable right detail slot. Only one detail target is active at a time.
- The composer is always reachable and owns slash commands, mentions, prompt history, attachments, runtime controls, send, steer, and stop.
- Tool output is summarized in the transcript and expanded in place or in the detail slot. Large output is never fully laid out by default.
- Markdown and syntax highlighting use gpui-component primitives where they meet behavior and accessibility needs. Kimini adds components only for Kimi-specific semantics.
- Mermaid and KaTeX parity uses one reusable on-demand rich-content renderer rather than one WebView per message. Rendered output is cached as a native image/vector surface with copyable source and accessible description; the helper unloads when idle.
- Terminal behavior uses a proven VT parser and a GPUI renderer. Kimini does not implement terminal escape parsing from scratch.
- Full source editing, language servers, and debugger UI are excluded; file preview, review, and external-editor handoff cover the Kimi Code Web product boundary.

### Browser architecture

- Browser Use is independent from the WRY legacy shell. Opening Browser Use launches an isolated Chromium profile and Browser Broker; closing it tears both down.
- Browser Broker exposes model tools through MCP and controls the same browser through Playwright/CDP. Kimini observes lifecycle, tabs, navigation, permissions, frames, and takeover state through authenticated local IPC.
- First delivery uses a visible companion Chromium window. This is sufficient for agent navigation and user observation without blocking the native GUI milestone.
- The next delivery can show CDP screencast frames in the GPUI detail slot and forward pointer/keyboard input. Full user takeover can promote the same target to its companion window when native frame input is insufficient.
- A bounded WRY child pane is retained for human preview and OAuth flows. It occupies a reserved rectangle; Kimini hides it before displaying overlapping GPUI menus or dialogs and explicitly bridges focus and shortcuts.
- On macOS, destroying the WRY child removes its WebContent process. WKWebView may retain pooled GPU and Networking helpers until the host exits; strict process-family teardown therefore belongs to the isolated Browser Broker or companion process.
- External Chrome uses a separate adapter and explicit consent because it exposes the user's profile. Full CDP and Computer Use have separate grants.
- Browser credentials are requested and filled by host-owned UI. Secret values never cross MCP or model-visible browser capture.

### Security and privacy

- Automatic discovery connects only to loopback. Any future remote-server mode requires an explicit origin and transport-security design.
- Server and browser credentials are redacted at the logging boundary. Diagnostics operate on structured fields with allowlisted safe values.
- External navigation, file opening, downloads, clipboard, media devices, location, popups, and user-profile browser access each require a visible policy decision.
- Approvals and questions are daemon-authoritative. Kimini does not infer permission from presentation state or silently retry a rejected privileged action.
- Browser helper IPC uses a per-launch secret and local endpoint; stale helpers cannot reconnect to a later Kimini process.

### Platform and dependency policy

- macOS 14+ is the first supported native platform and retains current dual-architecture packaging.
- Windows follows after the native core passes on macOS; GPUI input, AccessKit, menus, window behavior, and WRY/WebView2 integration are treated as release gates.
- Linux X11 can follow the same child-view route. Wayland in-window browser embedding waits for a supported backend; companion browser mode remains the fallback.
- GPUI and gpui-component track their current upstream Git sources through exact `Cargo.lock` revisions and are updated intentionally together. This is required because crates.io GPUI 0.2.2 predates the AccessKit integration used by Kimini.
- The G0 dependency set is GPUI/gpui-component/gpui-platform, Serde, `ureq`, `tungstenite`, `async-channel`, URL, and WRY. New rendering or parsing crates require a demonstrated missing capability.

### Delivery sequence and acceptance gates

1. **G0 — native proof:** two app bundles, GPUI shell, daemon connection, typed snapshot/WS path, session list, virtual conversation, cached streaming text, composer, stop, one approval/question flow, CJK input, AccessKit semantics, and a disposable WRY child pane. The implementation exists; packaged CJK, 1,000-turn, overlap, and matched p95 runs remain acceptance work.
2. **G1 — daily conversation:** workspaces, full session lifecycle, pagination/search, robust resync, Markdown/tool cards, attachments, runtime controls, settings, auth, notifications, and bilingual UI.
3. **G2 — coding parity:** file tree/search/preview, diff/git state, terminal, tasks/subagents, side chat, goals, skills, export, rich-content rendering, and complete approval/question variants.
4. **G3 — browser capability:** visible isolated Chromium plus MCP/Playwright/CDP, permissions, user takeover, browser lifecycle, then the in-app frame view and bounded WRY human-browsing pane.
5. **G4 — native release:** same-workload product measurements, compatibility fallback, packaging/signing, crash recovery, documentation, and release hardening while retaining the separate Kimini Web app.

G0 continues only when all of these hold on representative hardware:

- 1,000 mixed-height turns remain correctly anchored and interactive during sustained streaming.
- Chinese and Japanese composition, candidate placement, selection, clipboard, dead keys, and emoji work in the real packaged app.
- VoiceOver exposes useful roles, names, focus order, actions, and streaming announcements.
- Browser child-surface resize, focus, renderer cleanup, and overlap policy behave predictably in the bounded pane; pooled WKWebView helper residue is measured separately.
- Browser-off GUI physical-footprint targets are p95 ≤ 120 MiB for a short session and p95 ≤ 250 MiB for a long session; these are prototype gates, not published results.
- Browser-off idle CPU target is p95 ≤ 1% of one core; frame time is p95 ≤ 16.7 ms; input feedback is p95 ≤ 50 ms.
- Closing isolated Browser Use removes its process family and returns the client near its pre-browser baseline. The WRY human-preview pane is held to its documented WKWebView lifecycle instead of this stricter broker gate.

### Current G0 evidence

- Both signed ad-hoc macOS bundles build together: `Kimini.app` is 12 MiB and `Kimini Web.app` is 1.7 MiB on arm64.
- Protocol and pure application-state code currently reports 93.99% line, 89.89% region, and 97.56% function coverage. GUI, platform, process, and transport glue remain outside this numeric gate and use live scenarios.
- The local-daemon suite reads the real session list and atomic snapshot, then completes an authenticated v1 WebSocket handshake without mutating user sessions.
- A single-instance macOS accessibility-tree run exposed application, heading, status, list, button, article, and text-area semantics. Packaged CJK composition and streaming announcements remain open.
- The bounded WRY pane created WebContent, GPU, and Networking helpers only after opening. Closing removed WebContent; pooled helper residue lasted until host exit in that run.
- A short release-binary spot sample with both applications connected to the same daemon placed the native host around 87–97 MiB and the Web host around 99 MiB, plus roughly 317 MiB of WebKit helpers. This is diagnostic evidence only; it is not the matched p95 product benchmark.
- The 1,000-turn, sustained-streaming, packaged CJK, overlap, frame-time, input-latency, and matched process-family scenarios remain open, so G1 has not started.

## Testing Decisions

- Tests assert externally meaningful behavior: visible application state, emitted daemon/browser effects, accessibility semantics, recovery outcomes, and bounded resource behavior. They do not assert GPUI node arrangement or private helper calls.
- The primary seam is an `AppModel` scenario harness. A scenario feeds user intents, command results, snapshots, and ordered WebSocket frames into the reducer, then reads semantic view state and outbound effects. This one seam covers most product behavior without a live window or daemon.
- G0 recorded fixtures cover snapshot seeding, ordered streaming deltas, durable duplicates, cursor gaps, volatile offsets, approval/question restoration, completed-message reconciliation, and unknown events. Later-stage fixtures are added with the feature that consumes them.
- Protocol contract coverage parses representative session, snapshot, message, interaction, control, and event payloads into tolerant Rust DTOs. Field additions remain compatible; missing required fields fail loudly.
- The current loopback suite exercises discovery, session listing, a real snapshot, and a real authenticated WebSocket handshake without mutating session data. Prompt/abort and interaction mutation scenarios remain gated follow-up work.
- G0 GPUI acceptance reads the real macOS accessibility tree and invokes exposed buttons through AccessKit. Packaged CJK composition, streaming announcements, and virtual-list anchoring remain explicit manual/automation work before the gate closes.
- Browser coverage starts Browser Broker with an isolated test profile and a local test site, then covers navigation, DOM actions, screenshots/frames, permission denial, takeover exclusivity, crash recovery, credential redaction, and complete process cleanup.
- Performance scenarios are blank launch, short session, 1,000-turn long session, full scroll, sustained streaming, browser launch, active browser use, and browser close. Measurements include the complete Kimini client/browser helper process family and report the daemon separately.
- Prior art comes from Kimi Code Web's daemon-client, event-projector, event-reducer, event-batcher/resync, workspace-state, WebSocket lifecycle, turn-rendering, attachment, task-poller, and notification suites. Rust scenarios should preserve those behavioral cases instead of translating their internal structure line for line.
- Current Kimini daemon-discovery and language tests remain and are reused by both startup modes.

## Out of Scope

- Reimplementing or bundling the Kimi agent engine, model providers, or daemon.
- Replacing or removing Kimi Code Web.
- Electron, Tauri, or a WebView-rendered permanent native interface.
- A full IDE editor, language server UI, debugger, source-control mutation UI, or extension marketplace in the first native release.
- A general Kimini plugin SDK before two real browser/tool providers require the same extension contract.
- Whole-desktop Computer Use in G0-G3.
- User-profile external Chrome without separate consent and a dedicated adapter.
- Multi-daemon management, remote cloud daemon connections, collaboration, or mobile clients in the first release.
- Linux Wayland in-window WebView embedding in the first release.
- Pixel-identical reproduction of browser layout quirks; semantic, interaction, and design-system parity is required.
- Claims that native mode has won on memory, CPU, latency, or GPU use before the defined same-workload runs complete.

## Further Notes

- Source research was refreshed from Kimi Code `main` on 2026-07-18. The web client already proves the REST/WS projection, snapshot watermark, sequence/epoch recovery, event batching, and interaction semantics that native mode must preserve.
- Codex is used as a protocol, interaction, permission, and browser-boundary reference. Its inspected desktop shell is Electron/Chromium and provides no public native GUI implementation to copy.
- Zed proves GPUI's coding-workload fit and raw-window-handle escape hatch. Zed's browser MCP extensions prove external browser control; its unmerged WebView prototypes document child-surface z-order, focus, clipping, and Wayland limitations.
- Supporting evidence lives in [Native GUI Framework Selection](./native-gui-framework-selection.md) and [Codex Reference Architecture](./codex-reference-architecture.md).
- G1 starts only after the remaining G0 gates are recorded as passed or explicitly re-scoped with evidence.
