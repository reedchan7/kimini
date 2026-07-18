# Codex Reference Architecture for Kimini

## Scope and evidence level

This note separates two evidence surfaces.

### Public repository evidence

The public `openai/codex` repository was synchronized and audited at:

- branch: `main`
- commit: `bcdc695877cc46d1475ccea295b6eda6fe4171cb`
- commit date: `2026-07-18T00:06:49Z`
- subject: `Track inherited paginated rollout prefixes (#33930)`

Repository claims refer to source code, documentation, generated protocol types, and comments at that commit.

### Installed artifact evidence

The local installed product snapshot is `/Applications/ChatGPT.app`, bundle identifier `com.openai.codex`, version `26.715.21425`, build `5488`. Packaged files, bundled documentation, and one active process snapshot were inspected read-only. This evidence can establish what this exact installed build ships and was running; it cannot establish public source availability, every release's topology, or a general lifecycle guarantee. Local artifact paths and process observations may drift after an application update.

Claims are labeled as **confirmed**, **snapshot**, **inference**, or **unknown** where the distinction matters.

## Executive findings

1. **The public repository does not contain the Codex Desktop GUI source.** It contains the Rust CLI, TUI, core, app-server, protocol, SDKs, and launch/install glue for the separately distributed desktop application.
2. **Codex Desktop is Electron/Chromium on both observed platforms.** Public source directly confirms the Windows ASAR/Electron launch path. The installed macOS build independently contains `app.asar`, Electron internals, Renderer/GPU/Service helpers, and Chromium 150 through Codex Framework `150.0.7871.124`.
3. **The strongest reusable part is the app-server contract:** Thread -> Turn -> typed Item, incremental notifications, authoritative completed items, bidirectional approvals, pagination, subscriptions, and bounded backpressure.
4. **Browser capabilities are deliberately separated:** in-app browser UI, agent Browser Use, full CDP access, external-browser integration, and OS-level Computer Use are five independent policy gates.
5. **The public browser implementation is absent, while the installed build exposes a concrete capability contract.** Its shipped browser artifacts advertise `iab`, `extension`, and `cdp` backends, browser- and tab-scoped capabilities, screenshots, DOM snapshots, DOM/CUA input, Playwright, buffered CDP events, and a model-blind credential handoff.
6. **Codex Desktop is a protocol, interaction, and browser-boundary reference.** It is not evidence for a pure-Rust GUI. A native Kimini shell with an on-demand isolated browser module remains a distinct and defensible architecture.

## 1. What is public, and what the Desktop UI uses

### Confirmed public scope

The root README describes the project as Codex CLI and sends users to a separately installed Desktop experience (`README.md:1-8`). The JavaScript workspace contains only the CLI wrapper, Responses API proxy package, and TypeScript SDK (`pnpm-workspace.yaml:1-4`). The open Rust UI is `codex-tui`, whose manifest declares Ratatui and Crossterm (`codex-rs/tui/Cargo.toml:1-18,71-87`). That TUI is unrelated to the pixel GUI implementation.

`codex app` is installation and launch glue:

- macOS searches for `ChatGPT.app` or `Codex.app`, downloads a prebuilt DMG when absent, and opens `codex://threads/new?...` (`codex-rs/cli/src/desktop_app/mac.rs:8-41,67-79,101-154`).
- Windows detects the Store package and opens the same custom URL scheme (`codex-rs/cli/src/desktop_app/windows.rs:6-30,33-74`).

No Desktop view tree, styling system, browser-pane implementation, or GUI package is present in the tracked workspace.

### Confirmed Desktop runtime family

The Windows continuation launcher:

- locates the package-declared executable;
- refers to an "internal Electron shim";
- requires `resources/app.asar`;
- launches the executable with that ASAR path and the Codex URL (`codex-rs/tui/src/app/history_ui.rs:225-266`).

This is direct public-source evidence that the Windows Desktop distribution uses Electron packaging.

The installed macOS `26.715.21425` bundle independently contains:

- `Contents/Resources/app.asar`;
- Chromium 150 through Codex Framework `150.0.7871.124`, plus Electron internal modules;
- packaged `Codex (Renderer)`, `Codex (GPU)`, and `Codex (Service)` helpers;
- a main application process with Chromium renderer, GPU, and utility-service children in the observed active session.

Together these establish Electron/Chromium for the observed Windows and macOS distributions. They still do not reveal the renderer framework or make the ASAR's GUI implementation public source.

### What cannot be learned from this repository

The following remain **unknown** from public source:

- React or another renderer framework, state library, design system, and component hierarchy;
- DOM virtualization strategy, editor implementation, browser view embedding API, and accessibility implementation;
- production memory, CPU, GPU, startup, or long-thread measurements across representative workloads;
- whether future builds preserve the exact same helper and browser-process topology.

Kimini should not select a GUI toolkit by reverse-inferencing these missing details.

## 2. Desktop and `app-server`: process and protocol boundaries

### Confirmed protocol boundary

`codex app-server` is documented as the interface for rich clients (`codex-rs/app-server/README.md:1-4`). Its wire protocol is bidirectional JSON-RPC 2.0 with the `jsonrpc` header omitted. Supported transports are:

- default stdio with JSONL;
- WebSocket over a local Unix socket for the control plane;
- experimental TCP WebSocket;
- transport disabled (`codex-rs/app-server/README.md:20-42`).

Every connection performs `initialize` followed by `initialized`. A client then starts or resumes a thread, starts turns, consumes notifications, and answers server-initiated requests (`codex-rs/app-server/README.md:64-85`).

The same semantic contract can cross or avoid a process boundary:

- external clients use JSON over stdio/socket transports;
- CLI surfaces can embed app-server in-process over typed bounded channels while preserving the JSON-RPC response contract (`codex-rs/app-server-client/README.md:3-43` and `codex-rs/app-server-client/src/lib.rs:420-450`).

### Confirmed independent daemon topology

The experimental daemon manages a detached app-server for remote desktop and mobile clients. It stores a PID, exposes a Unix control socket, and supports lifecycle commands (`codex-rs/app-server-daemon/README.md:1-15,34-45,84-113`). The managed process is launched as `codex app-server --listen unix://`, optionally with remote control (`codex-rs/app-server-daemon/src/backend/pid.rs:412-422`).

### Installed Desktop process snapshot

The public `codex app` launcher only installs or opens the separately distributed application, so the repository alone does not document its production process topology.

In the inspected active macOS session, the Electron main process had direct children for a Renderer, a GPU process, utility services used for network/storage work, and the bundled Rust executable launched as `codex ... app-server`. Additional browser/code-mode Node and Rust descendants appeared under active sessions. This directly confirms a separate Rust app-server boundary in this local snapshot.

This observation should not be generalized into a lifecycle guarantee: it does not establish when the child starts, whether it survives main-window closure, which transport is used, or whether all future builds use the same parentage. The safe architectural lesson for Kimini is the stable backend/UI contract plus independently recoverable process boundaries.

## 3. Thread, turn, item, and streaming UI model

### Domain model

The protocol defines three top-level interaction primitives (`codex-rs/app-server/README.md:64-72`):

```text
Thread
  -> Turn
       -> ThreadItem[]
```

`ThreadItem` is a tagged union for user messages, agent messages, plans, reasoning, commands, file changes, MCP calls, dynamic tools, collaboration, web search, image viewing/generation, and lifecycle markers (`codex-rs/app-server-protocol/src/protocol/v2/item.rs:223-391`).

This is a useful GUI boundary: the view layer renders stable product concepts instead of raw model-provider events.

### Incremental lifecycle

The canonical lifecycle is (`codex-rs/app-server/README.md:1403-1442`):

```text
turn/started
  item/started
  zero or more item-specific deltas
  item/completed
turn/completed
```

Agent text deltas carry `threadId`, `turnId`, `itemId`, and `delta` (`codex-rs/app-server-protocol/src/protocol/v2/item.rs:1320-1329`). `item/completed` carries the complete final `ThreadItem` and is the authoritative item state (`codex-rs/app-server-protocol/src/protocol/v2/item.rs:1299-1309`). Current `turn/completed` notifications deliberately carry `items: []` with `itemsView: notLoaded`; clients must retain items built from item events (`codex-rs/app-server/src/bespoke_event_handling.rs:1257-1278`).

Core events are projected into stable app-server notifications by an explicit mapping layer (`codex-rs/app-server-protocol/src/protocol/event_mapping.rs:25-37,359-416`). Kimini should keep the same separation: daemon event -> protocol DTO -> pure GUI reducer -> render model.

### Recovery model and its limit

Codex supports resume plus durable turn/item pagination. Clients can resume without eagerly loading all turns, request an initial page, and select `notLoaded`, `summary`, or `full` item detail (`codex-rs/app-server-protocol/src/protocol/v2/thread.rs:387-483,1384-1413`; `codex-rs/app-server/README.md:321-327,513-519`).

The live notification envelope contains `emittedAtMs`, but no general sequence number, epoch, or replay cursor (`codex-rs/app-server-protocol/src/protocol/common.rs:1742-1758`). Reconnection recovery therefore depends on durable snapshots/pages plus new live events. Kimi's existing sequence/epoch journal should be retained where available because it provides a stronger gap-detection contract.

## 4. Approval and user-input flows

App-server sends approvals as server-initiated JSON-RPC requests. The client responds on the same request ID. Command, file-change, permissions, dynamic-tool, MCP elicitation, and user-input requests are distinct protocol variants (`codex-rs/app-server-protocol/src/protocol/common.rs:1482-1519`).

For command and file approvals, the documented UI flow is (`codex-rs/app-server/README.md:1497-1524`):

1. render `item/started` immediately;
2. attach the approval request to that `threadId` / `turnId` / `itemId`;
3. send the selected decision;
4. clear pending UI on `serverRequest/resolved`;
5. replace provisional state with authoritative `item/completed`.

The backend retains each pending request with a callback and its thread association (`codex-rs/app-server/src/outgoing_message.rs:287-350`). A resumed connection receives outstanding requests again before normal idle continuation (`codex-rs/app-server/src/outgoing_message.rs:353-371`; `codex-rs/app-server/src/request_processors/thread_lifecycle.rs:715-723`). Resolution emits `{threadId, requestId}` so every surface can remove stale prompts (`codex-rs/app-server/src/request_processors/thread_lifecycle.rs:772-795`; `codex-rs/app-server-protocol/src/protocol/v2/notification.rs:50-56`).

This is directly applicable to Kimini. Approval state should live in the protocol projection, survive view recreation, and never depend solely on a modal widget's local state.

## 5. Browser, Browser Use, CDP, and Computer Use

### Confirmed capability separation

The Rust feature model distinguishes five requirements-owned gates (`codex-rs/features/src/lib.rs:177-196`):

| Gate | Product boundary |
| --- | --- |
| `InAppBrowser` | Whether a browser pane may exist in the Desktop UI |
| `BrowserUse` | Whether the agent may use browser automation |
| `BrowserUseFullCdpAccess` | Whether automation may access the full CDP surface |
| `BrowserUseExternal` | Whether external browsers may be controlled |
| `ComputerUse` | Whether Codex may control desktop applications and OS UI |

At this commit they are stable and enabled by default before requirements override (`codex-rs/features/src/lib.rs:1133-1162`). Enterprise requirements separately expose `computerUse.allowLockedComputerUse` (`codex-rs/app-server-protocol/src/protocol/v2/config.rs:371-418`).

The repository's bundled OpenAI-docs skill describes Browser Use/in-app browser as Codex-controlled web testing, the Chrome extension as using the user's Chrome profile, and Computer Use as controlling desktop/OS UI (`codex-rs/skills/src/assets/samples/openai-docs/SKILL.md:118`). These are separate trust boundaries and should remain separate in Kimini.

### Public extension/plugin seam

Tool discovery names `chrome@openai-bundled` and `computer-use@openai-bundled` as special discoverable plugins (`codex-rs/core-plugins/src/discoverable.rs:17-48`). Their implementation bundles are absent from the public tree.

The public item protocol has no browser-frame or CDP-event item. It does expose generic `mcpToolCall` and `dynamicToolCall` items with wire-shaped JSON results (`codex-rs/app-server-protocol/src/protocol/v2/item.rs:300-333`; `codex-rs/app-server-protocol/src/protocol/v2/mcp.rs:128-142`). Dynamic tools are a reusable pattern for executing typed client-owned capabilities, but the public source does not prove that Browser Use itself is transported through that item type. `webSearch` is a separate model web-search item and should not be confused with an interactive browser (`codex-rs/app-server/README.md:1421-1433`).

### Installed browser capability contract

The installed `26.715.21425` bundle ships proprietary `browser`, `chrome`, and `computer-use` plugin artifacts. Artifact references below are relative to `Contents/Resources/plugins/openai-bundled/plugins/browser/`. The packaged browser API exposes a backend-adapter model:

```text
agent.browsers
  -> Browser { type: iab | extension | cdp }
       -> browser-scoped capability collection
       -> tabs
            -> tab-scoped capability collection
            -> screenshot / DOM snapshot
            -> DOM CUA / coordinate CUA / Playwright
            -> optional raw CDP / browserAuth / page-assets adapters
```

This contract is explicit in `browser/docs/api.json`: browser discovery reports `type: "iab" | "extension" | "cdp"` and separate browser/tab capability arrays (lines 55-63); `Browser.capabilities` and `Tab.capabilities` are independently discoverable collections (lines 73-82 and 235-244); and tabs expose CUA, DOM CUA, Playwright, screenshots, and DOM snapshots (lines 266-313, 385-400, and 591-603). The division is valuable for Kimini because each backend can advertise only what it safely implements instead of forcing one lowest-common-denominator browser API.

The packaged runtime also includes `cua_node` with Playwright and Playwright Core `1.57.0`. This proves that the installed product carries a Node/Playwright automation substrate; it does not disclose the Desktop pane's rendering or compositing implementation.

Two installed capability designs are especially reusable:

- **Buffered CDP events:** `cdp.readEvents()` uses an `afterSequence` cursor, bounded page size, `hasMore`, and `truncated` when old events were evicted (`browser/docs/capabilities/tab/cdp.md:1-26`). This is a browser-local journal contract suitable for console/network/event streams. It should remain separate from the conversation journal.
- **Model-blind authentication:** `browserAuth.request()` pauses for credentials in a secure host UI, fills the validated form, and never returns secret values to the caller (`browser/docs/capabilities/tab/browserAuth.md:1-16,145-214`). Kimini should treat this as a privileged UI-owned handoff, outside model-visible DOM, screenshots, tool output, logs, and chat history.

The `extension` backend and packaged Chrome plugin confirm that controlling the user's external Chrome profile is a separate adapter from the in-app browser. The packaged Computer Use helper is another distinct capability and process boundary.

### What remains unavailable

No current public tracked source implements:

- a Playwright client or server;
- Chromium process launch/profile management;
- CDP session routing, frame capture, input forwarding, or target ownership;
- browser-pane embedding or compositing;
- user takeover, browser permission UI, or external Chrome-extension transport;
- the mapping between browser automation state and the visible Desktop pane.

The installed artifacts reveal the shipped capability surface and backend taxonomy, while the following remain **unknown**: the GUI's browser-view component, browser-surface embedding/compositing, exact IPC between Electron and browser workers, takeover-state ownership, profile lifetime rules, and internal source architecture.

## 6. Performance and concurrency patterns worth adopting

### Installed footprint snapshot

The inspected application bundle is `1.4G` on disk. Major shipped components include `app.asar` at `203M`, `cua_node` at `374M`, and Codex Framework at `361M`. The active session showed the Electron main process, Renderer, GPU process, network/storage utility services, Rust app-server, and optional browser/code-mode descendants.

These figures demonstrate multi-process and distribution-size cost for this installed build. They are not a memory benchmark: RSS was intentionally not summed because process sharing, workload, browser state, and measurement timing would make that number misleading. Kimini needs its own cold/short/long/streaming/browser-on benchmark over the complete process tree.

### Bounded queues and explicit overload

Transport queues are bounded at 128 messages. Saturated request ingress returns retryable JSON-RPC error `-32001` instead of growing memory without limit (`codex-rs/app-server-transport/src/transport/mod.rs:21-24,217-255`; `codex-rs/app-server/README.md:49-53`).

The in-process client divides events into:

- lossless: transcript/reasoning/plan deltas and authoritative item/turn completion;
- best effort: command output and cosmetic progress.

Best-effort events may be dropped under pressure and summarized with a lag marker, while lossless events wait for capacity (`codex-rs/app-server-client/src/lib.rs:115-150,162-237`). Kimini should use the same semantic tiers, with byte-based caps for large command and browser payloads.

### Serialize only the resource being mutated

Requests declare scopes such as global, thread, process, file-watch, and MCP OAuth (`codex-rs/app-server-protocol/src/protocol/common.rs:114-125`). Each resource key has its own FIFO; consecutive shared reads can run together, while unrelated keys remain concurrent (`codex-rs/app-server/src/request_serialization.rs:18-103,143-225`; `codex-rs/app-server/src/message_processor.rs:822-860`).

For Kimini this means one reducer/command lane per Kimi session and browser target, without a global application mutex.

### Subscribe only to active work

Clients can suppress exact high-frequency notification methods per connection (`codex-rs/app-server/README.md:1350-1372`). After the final subscriber leaves, a thread remains warm only while active and unloads after 30 idle minutes (`codex-rs/app-server/README.md:466-490`; `codex-rs/app-server/src/request_processors/thread_lifecycle.rs:1-80`).

Kimini should subscribe to full-fidelity live events only for visible or background-running sessions, page cold history, and bound warm-session retention by both time and memory.

### Coalesce rendering separately from event ingestion

The open TUI provides two useful rendering ideas:

- frame requests are coalesced and capped at 120 FPS (`codex-rs/tui/src/tui/frame_requester.rs:1-9,70-125`; `codex-rs/tui/src/tui/frame_rate_limiter.rs:1-37`);
- streaming uses smooth and catch-up modes based on queue depth and age, with hysteresis to avoid oscillation (`codex-rs/tui/src/streaming/chunking.rs:1-65,82-116,176-209`).

Its stable-region/mutable-tail model also handles incomplete Markdown and tables (`codex-rs/tui/src/streaming/controller.rs:1-36`). The exact TUI implementation fully re-renders accumulated Markdown on each committed delta (`codex-rs/tui/src/streaming/controller.rs:74-93`), so Kimini should adopt the state model while using incremental block parsing and visible-range layout.

## 7. Recommended Kimini interpretation

Codex changes the reference model, not the native-GUI decision:

```text
Kimini Native GUI
├── native conversation, diff, terminal, approval, and settings UI
├── Kimi protocol adapter and pure event projector
├── existing Kimi daemon as an independent recoverable process
└── UI-owned Browser Broker
    ├── iab adapter -> embedded child surface + isolated profile
    ├── headless adapter -> existing Kimi MCP/browser tools
    ├── extension adapter -> external Chrome + separate consent
    └── Computer Use adapter -> OS control + strongest consent
```

Recommended borrowings:

1. Thread/turn/item IDs and an authoritative final-item replacement model.
2. Server-initiated approvals with replay and explicit resolution notifications.
3. Durable snapshot/page bootstrap followed by live deltas.
4. Bounded semantic event tiers and resource-keyed request serialization.
5. A capability-advertising browser broker with independent gates for pane, automation, full CDP, external profiles, and OS control.
6. Cursor-based bounded journals for high-volume browser events, including explicit truncation recovery.
7. Host-owned secret entry that never crosses a model-visible channel.
8. Render-frame coalescing and adaptive catch-up under streaming pressure.

Recommended non-borrowings:

1. Do not make Electron the native-shell baseline merely because Codex ships Electron/Chromium.
2. Do not couple browser availability to making the entire app a browser renderer.
3. Do not replace Kimi's stronger sequence/epoch replay with Codex's timestamp-only live envelope.
4. Do not copy the TUI's whole-stream Markdown re-render into a long-lived GPU GUI.
5. Do not merge browser authentication, external-profile control, and OS Computer Use into one permission.
6. Do not guess at Codex's closed GUI source, browser compositing, or general process lifecycle from a single installed snapshot.

## 8. Implementation order and exit gates

### G0: protocol projector and native long-chat surface

Implement the Kimi session projector, typed render model, virtualized long-thread view, streaming tail, approval state, reconnect/replay, and daemon lifecycle without an agent-controlled browser. A disposable bounded WRY pane may coexist in G0 to prove human preview/OAuth child-surface mechanics; it does not own Browser Use state.

**Exit gate:** blank, short, long, scrolling, active streaming, approval, cancellation, and reconnect fixtures converge to the same authoritative session state; long-thread interaction stays responsive under the native GUI resource targets defined by the main Kimini plan.

### G1: UI-owned browser broker with existing Kimi automation

Introduce stable browser/target IDs, backend and capability descriptors, per-target command lanes, permission state, and bounded screenshot/event payloads. Route the current Kimi MCP or headless browser tools through this broker while keeping their execution out of the GUI process.

**Exit gate:** one agent turn can open, inspect, act, stream progress, cancel, recover from worker failure, and present an auditable tool result through the broker; browser-off sessions pay no resident browser-process cost.

### G2: embedded in-app browser child surface and takeover

Add the `iab` adapter as an on-demand child web surface with an isolated profile. Make the UI broker the sole owner of tabs, automation leases, visibility, user takeover, and model-blind authentication handoff.

**Exit gate:** the visible tab and automation target cannot diverge; agent-to-user and user-to-agent takeover is explicit and race-free; crash/restart preserves conversation state and fails the browser target cleanly; credentials never enter model-visible events or logs.

### G3: external Chrome adapter

Add a separate `extension` adapter for an existing Chrome profile. Preserve the same browser/target contract while advertising only supported capabilities and requiring distinct consent.

**Exit gate:** tab discovery/claim/release, disconnect recovery, and profile boundaries work without granting in-app-browser permissions to external Chrome or leaking external browsing state into unrelated sessions.

### G4: Computer Use

Add OS-level screen/input control as a separately packaged, separately gated capability after browser-only control is mature.

**Exit gate:** explicit scope and emergency stop exist; locked-screen and sensitive-input policies are enforced; every action has an attributable target and lifecycle; disabling Computer Use leaves browser and native chat operation unchanged.

## Bottom line

The public Codex repository validates a split architecture: a rich client consumes a stable agent protocol, while privileged capabilities are independently gated. The installed macOS product adds concrete evidence of an Electron/Chromium shell, a separate Rust app-server, and browser adapters for in-app, external-extension, and CDP control.

For Kimini, the higher-value target is a native Rust primary UI with Codex-like protocol discipline and an on-demand, isolated browser subsystem. Web technology can remain inside the capability that renders and controls web content, while the conversation and coding interface avoids a permanent browser process tree.
