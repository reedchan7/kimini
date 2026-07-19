<div align="center">

<img src="docs/brand/exports/app-icon-128.png" width="112" height="112" alt="Kimini app icon"/>

# Kimini

**A native macOS GUI for Kimi Code — with the Web experience one app away.**

[English](README.md) · [简体中文](README_CN.md)

<a href="https://github.com/reedchan7/kimini/releases/latest"><img src="https://img.shields.io/badge/version-0.3.0-4A90D9?style=flat-square&logo=github" alt="Version 0.3.0"/></a>
<a href="#compatibility-and-release-facts"><img src="https://img.shields.io/badge/core%20coverage-97.03%25-brightgreen?style=flat-square&logo=rust" alt="Core coverage 97.03%"/></a>
<a href="#compatibility-and-release-facts"><img src="https://img.shields.io/badge/local%20tests-186%20passed-brightgreen?style=flat-square&logo=rust" alt="186 local tests passed"/></a>
<img src="https://img.shields.io/badge/platform-macOS%2014%2B-black?style=flat-square&logo=apple&logoColor=white" alt="macOS 14+"/>
<a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="MIT license"/></a>

</div>

![Kimini 0.3 native conversation UI](docs/screenshots/kimini-native-overview.png)

Kimini 0.3 turns the local [Kimi Code](https://github.com/MoonshotAI/kimi-code)
workflow into a focused GPUI desktop application. It connects directly to the
Kimi daemon, keeps your existing sessions, and leaves the official Web workflow
available through a separate companion app.

## Choose your app

| | **Kimini** | **Kimini Web** |
|---|---|---|
| Best for | Daily native workflow | Browser UI compatibility |
| Interface | GPUI + Metal | Kimi Code Web in system WKWebView |
| Connection | Typed REST + WebSocket | Daemon-served Web UI |
| Renderer | No permanent browser renderer | WebKit process family |
| Bundle | **14.1 MiB** | **1.8 MiB** |

Install both. They run side by side against the same local daemon and session
store.

## Why Kimini

- **Native where it matters.** Conversation, composer, sessions, settings, and
  coding surfaces are rendered with GPUI and Metal.
- **Your Kimi workflow stays intact.** Kimini discovers or starts the local
  daemon and uses its authenticated `/api/v1` REST and WebSocket contracts.
- **Web compatibility stays close.** Kimini Web preserves the daemon-served
  browser interface for features that still need exact Web behavior.
- **Coding work is first class.** Files, search, previews, git state, tasks,
  skills, goals, side chats, approvals, prompt queues, and terminal tabs live in
  one desktop workspace.
- **Browser cost is on demand.** The native app creates its human-preview
  WKWebView only when you open Browser and destroys the view when closed.

Compared with a broad agent environment such as
[Codex](https://openai.com/index/introducing-the-codex-app/), Kimini is
purpose-built for people already using Kimi Code: one daemon, one session
history, and a choice of native or Web desktop surfaces. Model quality and
cross-product performance are outside this comparison.

## Quick start

Requires macOS 14+ and Kimi Code.

```sh
curl -fsSL https://code.kimi.com/kimi-code/install.sh | bash
kimi login
```

Download the matching architecture from
[Releases](https://github.com/reedchan7/kimini/releases/latest):

- `Kimini-<version>-macos-<arch>` — native GPUI app
- `Kimini-Web-<version>-macos-<arch>` — Web compatibility app

Current builds are ad-hoc signed. On first launch, right-click the app and
choose **Open** if macOS blocks it.

Kimini reads `~/.kimi-code/server/lock` and `server.token`, health-probes the
daemon, and starts `kimi server run` when needed. Credentials stay in request
headers or the WebSocket subprotocol; they are not placed in browser URLs.

## Native experience

The 0.3 release includes session creation, search, rename, archive/restore,
fork, compact, undo, attachments, streaming, thinking and tool traces,
approvals and questions, runtime modes, files, tasks, skills, goals, side chat,
terminal tabs, authentication, English/Chinese UI, themes, and keyboard-first
navigation.

<details>
<summary><strong>Settings, appearance, language, account, and agent defaults</strong></summary>

![Kimini native settings](docs/screenshots/kimini-native-settings.png)

</details>

| Shortcut | Surface |
|---|---|
| `⌘⇧E` | Files |
| `⌘⇧K` | Skills |
| `⌘J` | Terminal |
| `⌘⇧T` | Tasks |

## Compatibility and release facts

- Direct daemon integration uses the self-described Kimi Code REST and
  WebSocket protocols; it does not scrape CLI/TUI output.
- Kimini Web remains an independent application and uses the same sessions and
  authentication as `kimi web`.
- The native terminal prefers the daemon backend and retains a local Rust PTY
  fallback when the packaged daemon cannot load its PTY module.
- Both 0.3.0 arm64 app bundles build and run together. The release bundles are
  14.1 MiB native and 1.8 MiB Web on the current build machine.
- Protocol and pure state logic currently have **97.03% line coverage**. The
  local native/Web release suite contains **186 automated tests**.

The native app is an early release. Packaged CJK composition, long-session
performance, streaming accessibility, rich media, and complete interactive
terminal behavior are still being hardened. Matched CPU and memory results will
be published after the full process-family benchmark is complete.

## Build from source

```sh
make run          # native app
make run-web      # Web compatibility app
make apps         # both .app bundles in dist/
make package-all  # both architectures, DMG + zip
```

See the [native GUI specification](docs/native-gui-spec.md) and
[framework decision](docs/native-gui-framework-selection.md) for deeper design
and performance methodology.

---

Community-built and independent. Kimi and Kimi Code are products of Moonshot
AI. Kimini is not affiliated with Moonshot AI.

[MIT](LICENSE) · [Issues](https://github.com/reedchan7/kimini/issues) · Built
with [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui),
[gpui-component](https://github.com/longbridge/gpui-component),
[wry](https://github.com/tauri-apps/wry), and
[tao](https://github.com/tauri-apps/tao).
