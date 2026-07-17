<div align="center">

<img src="docs/brand/exports/app-icon-128.png" width="128" height="128" alt="Kimini app icon"/>

# Kimini

**The lightest way to browse.**

A ~1 MB native macOS app for [Kimi Code Web](https://github.com/MoonshotAI/kimi-code) —
one window, one system WebView, no bundled browser.

<a href="https://github.com/reedchan7/kimini/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/reedchan7/kimini/ci.yml?branch=main&style=flat-square&label=CI&logo=github" alt="CI"/></a>
<a href="https://github.com/reedchan7/kimini/releases/latest"><img src="https://img.shields.io/github/v/release/reedchan7/kimini?style=flat-square&logo=github&color=4A90D9" alt="Release"/></a>
<a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="License MIT"/></a>
<img src="https://img.shields.io/badge/platform-macOS%2014%2B-black?style=flat-square&logo=apple&logoColor=white" alt="macOS 14+"/>

**English** · [中文](README_CN.md)

</div>

Kimini gives `kimi web` its own window, Dock icon, `⌘Tab` entry and menu
bar — without shipping a browser. The Rust host is a ~0.9 MB binary;
rendering, fonts and IME stay with the WebKit already in macOS, so the whole
app is ~1.2 MB where an Electron wrapper would be 100 MB+. Navigation is
loopback-only: external links open in your default browser, and the page
gets no JS bridge.

*The name is just **Kimi** + **mini**.*

## Install

**Requires:** macOS 14+ · the [Kimi Code](https://github.com/MoonshotAI/kimi-code) CLI (`npm install -g @moonshot-ai/kimi-code`)

Download from [**Releases**](https://github.com/reedchan7/kimini/releases/latest)
— `aarch64` for Apple Silicon, `x86_64` for Intel — and drag **Kimini** into
**Applications**.

> [!NOTE]
> Builds are ad-hoc signed, so Gatekeeper blocks the first open:
> right-click → **Open**, or
> `xattr -dr com.apple.quarantine /Applications/Kimini.app`

```sh
# Zero-config — finds (or starts) the local kimi daemon and signs in:
open -a Kimini

# Optional — connect to an explicit URL instead:
open -na Kimini --args 'http://127.0.0.1:58627/#token=<daemon-token>'
```

## Usage

| | |
|---|---|
| `⌘,` | Settings — host UI language (English / 简体中文) |
| `⌘R` | Reload |
| `⌘[` / `⌘]` | Back / Forward |

Start URL: CLI argument → `$KIMINI_URL` → auto-discovery
(`~/.kimi-code/server/lock` + `server.token`, starting `kimi server run` when needed).
Language: `$KIMINI_LANG` (`en` / `zh`) → saved preference → system locale.

## Build from source

```sh
make app            # → dist/Kimini.app   (Rust 1.85+)
make install-app    # → ~/Applications
make help           # everything else: run, lint, dmg, package-all, publish-release
```

## Notes

- Loopback origins only (`127.0.0.1` / `::1` / `localhost`); devtools off in release.
- Not notarized yet; IME in the bundled `.app` not fully re-verified; macOS only.

---

[MIT](LICENSE) · Built on [wry](https://github.com/tauri-apps/wry) /
[tao](https://github.com/tauri-apps/tao) /
[muda](https://github.com/tauri-apps/muda) ·
Unofficial project, not affiliated with Moonshot AI.
