<div align="center">

<img src="docs/brand/exports/app-icon-128.png" width="128" height="128" alt="Kimini app icon"/>

# Kimini

**Native Kimi Code, with the Web fallback intact.**

A Rust 2024 desktop client for [Kimi Code](https://github.com/MoonshotAI/kimi-code).

<a href="https://github.com/reedchan7/kimini/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/reedchan7/kimini/ci.yml?branch=main&style=flat-square&label=CI&logo=github" alt="CI"/></a>
<a href="https://github.com/reedchan7/kimini/releases/latest"><img src="https://img.shields.io/github/v/release/reedchan7/kimini?style=flat-square&logo=github&color=4A90D9" alt="Release"/></a>
<a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="License MIT"/></a>
<img src="https://img.shields.io/badge/platform-macOS%2014%2B-black?style=flat-square&logo=apple&logoColor=white" alt="macOS 14+"/>

</div>

Kimini ships two independent applications that can be installed and run side
by side:

| Application | Purpose | Permanent renderer |
|---|---|---|
| **Kimini** | Native Kimi Code GUI | GPUI / Metal |
| **Kimini Web** | Complete compatibility surface | System WKWebView |

Both applications discover the same local Kimi daemon and use the same session
data. The native client talks directly to the daemon's typed REST and WebSocket
contracts; it does not wrap Kimi Code Web or scrape CLI/TUI output.

> [!IMPORTANT]
> The native application is a functional alpha. Its end-to-end G0 path and
> selected G1/G2 product surfaces are implemented, including session lifecycle,
> grouped tool traces, files, tasks, goals, skills, side chat, and daemon-backed
> terminals. Formal long-session, packaged CJK, accessibility, and matched
> performance acceptance remain open. Kimini Web stays available as the complete
> compatibility surface throughout native parity work.

## Architecture

```text
Kimi daemon
  ├── REST snapshots and commands
  └── journaled WebSocket events
          ↓
protocol → model reducer → cached presentation → GPUI views
    ↑             ↓
 typed API   user commands

On demand only:
GPUI browser slot → bounded WRY child WKWebView
Future agent use  → isolated Chromium browser broker / MCP / CDP
```

The source is split by responsibility:

- `src/protocol/` — tolerant wire DTOs and cursor/control contracts.
- `src/model/` — deterministic session reducer with no GUI or network code.
- `src/api/` — local REST client and bounded WebSocket event worker.
- `src/daemon/` — discovery, health probing, token loading, and startup.
- `src/native/` — GPUI composition, presentation cache, views, and browser host.
- `src/legacy_web/` — isolated Kimini Web compatibility application.

## Native alpha

The current native path includes:

- zero-configuration local daemon discovery and startup;
- typed session list, snapshot, prompt, abort, approval, and question requests;
- paginated session search plus create, rename, archive, restore, fork, compact,
  undo, and diagnostic export flows;
- sequence/epoch cursor handling, duplicate suppression, and authoritative
  snapshot reload after cursor gaps or socket closure;
- UTF-16 stream offsets, step-relative stream resets, subagent isolation, and
  unknown-event tolerance;
- bounded lossless event delivery from a dedicated WebSocket worker;
- variable-height conversation virtualization and cached transcript projection;
- native composer with attachments, slash commands, skills, queued prompts,
  steering, runtime modes, goals, streaming output, approvals, and multi-part
  questions;
- compact grouped tool traces and a dedicated thinking-preview pane modeled on
  the Kimi Code Web information hierarchy;
- workspace file browsing, search, previews, git state, and diff summaries;
- background task/subagent rosters, isolated BTW side chats, and daemon-backed
  terminal tabs with VT output, replay, resize, command input, and close;
- managed authentication state and English/Simplified Chinese host UI;
- AccessKit roles, labels, focus traversal, and native CJK input plumbing;
- a rectangular WRY child view that is created only after an explicit browser
  action and destroyed when closed.

The embedded browser is for human preview and OAuth. Deterministic model-driven
browsing remains a separate future subsystem with an isolated Chromium profile,
MCP/CDP control, explicit takeover, and independent permissions.

WKWebView has an important lifecycle constraint: closing the child view removes
its WebContent process, while macOS may retain pooled GPU and Networking helpers
until the host exits. Work that requires complete browser-process teardown must
run in the isolated browser companion process.

## Install

**Requires:** macOS 14+ and the
[Kimi Code](https://github.com/MoonshotAI/kimi-code) CLI.

```sh
npm install -g @moonshot-ai/kimi-code
```

Release assets are named separately:

- `Kimini-<version>-macos-<arch>` — native GPUI application.
- `Kimini-Web-<version>-macos-<arch>` — Web compatibility application.

Builds are currently ad-hoc signed. The first launch may require right-clicking
the app and choosing **Open**.

## Run from source

```sh
make run                         # Native Kimini
make run-web                     # Kimini Web, automatic daemon discovery
make run-web URL='http://127.0.0.1:58627/#token=<daemon-token>'

# Open a human-browser URL with the native app for integration work
KIMINI_BROWSER_URL='https://example.com' make run
```

Native startup resolves the daemon from `~/.kimi-code/server/lock` and
`server.token`, health-probes it, and starts `kimi server run` when needed.
Bearer tokens remain in headers or the WebSocket subprotocol and never enter
the native browser URL, logs, or rendered state.

## Build and package

```sh
make build        # Native debug binary
make build-web    # Web compatibility debug binary
make apps         # dist/Kimini.app + dist/Kimini Web.app
make package-all  # both apps, both macOS architectures, DMG + zip
make lint
make test
make coverage-core
```

`coverage-core` enforces at least 90% line coverage for the protocol and pure
application-state core. Platform glue is exercised through real daemon, real
window, accessibility-tree, browser-lifecycle, and packaged-app scenarios.

## Kimini Web measured footprint

The following measurements apply to the **Kimini Web** compatibility app, not
the native G0 client. Controlled runs used an M5 Max / macOS 26.5.2, Kimini Web
0.1.0, Chrome 150.0.7871.128, the same local Kimi daemon and session data, and a
roughly 1440 × 900 content area. Samples were collected at 1 Hz over 30–45 s
windows for the complete client process family; the shared daemon was excluded.

**Memory** — physical footprint, MiB, median / P95:

| Scenario | Kimini Web | Chrome | Delta |
|---|---:|---:|---:|
| Idle · long session · 120 Hz | **501 / 501** | 517 / 667 | -3% / -25% |
| Scroll · long session · 120 Hz | **629 / 885** | 716 / 908 | -12% / -3% |
| Streamed response · 120 Hz | **520 / 522** | 622 / 798 | -16% / -35% |

**CPU** — process-family sum, percent of one core, mean / P95:

| Scenario | Kimini Web | Chrome | Delta mean |
|---|---:|---:|---:|
| Idle · long session · 120 Hz | 11.1 / 18.2 | **7.0 / 14.5** | +60% |
| Scroll · long session · 120 Hz | 18.7 / 37.9 | **11.0 / 22.6** | +70% |
| Streamed response · 120 Hz | 19.0 / 33.4 | **13.0 / 31.5** | +47% |
| Idle · default view · 120 Hz | **6.1 / 10.3** | 12.8 / 30.1 | -53% |
| Idle · default view · 60 Hz | **3.7 / 6.2** | 10.2 / 25.2 | -64% |

The long-session and default-view rows differ because the web page has a
persistent animation and repaint cost scales with view complexity and refresh
rate. These figures motivated the native renderer; they are not evidence for a
native performance win. Native results will be published only after matched
short-, long-, streaming-, and browser-lifecycle runs cover every process.

## Notes

- Native GUI dependencies track Zed GPUI and gpui-component through exact
  `Cargo.lock` revisions because current AccessKit support is newer than the
  crates.io GPUI release.
- `wry 0.55` and `tao 0.34` remain paired for the Web compatibility app.
- macOS is the first target. Windows and Linux require separate input,
  accessibility, child-surface, and packaging acceptance.
- The project is unofficial and is not affiliated with Moonshot AI.

---

[MIT](LICENSE) · Built with
[GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui),
[gpui-component](https://github.com/longbridge/gpui-component),
[wry](https://github.com/tauri-apps/wry),
[tao](https://github.com/tauri-apps/tao), and
[muda](https://github.com/tauri-apps/muda).
