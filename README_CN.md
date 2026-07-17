<div align="center">

<img src="docs/brand/exports/app-icon-128.png" width="128" height="128" alt="Kimini 应用图标"/>

# Kimini

**最轻的方式打开浏览。**

为 [Kimi Code Web](https://github.com/MoonshotAI/kimi-code) 打造的 ~1 MB
macOS 原生应用——一个窗口、一个系统 WebView、零内置浏览器。

<a href="https://github.com/reedchan7/kimini/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/reedchan7/kimini/ci.yml?branch=main&style=flat-square&label=CI&logo=github" alt="CI"/></a>
<a href="https://github.com/reedchan7/kimini/releases/latest"><img src="https://img.shields.io/github/v/release/reedchan7/kimini?style=flat-square&logo=github&color=4A90D9" alt="Release"/></a>
<a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="License MIT"/></a>
<img src="https://img.shields.io/badge/platform-macOS%2014%2B-black?style=flat-square&logo=apple&logoColor=white" alt="macOS 14+"/>

[English](README.md) · **中文**

</div>

Kimini 让 `kimi web` 拥有独立窗口、Dock 图标、`⌘Tab` 切换和原生菜单栏，
而且不用附带一个浏览器。Rust 宿主只有约 0.9 MB；渲染、字体与输入法全部
交给 macOS 自带的 WebKit，整个应用约 1.2 MB——同样的事换 Electron 要
100 MB 起步。导航仅限回环地址：外链一律在系统默认浏览器打开，页面也拿
不到任何 JS 桥。

*名字就是 **Kimi** + **mini**。*

## 安装

**环境要求：** macOS 14+ · 本机已运行 [Kimi Code](https://github.com/MoonshotAI/kimi-code) 守护进程（`kimi web`）

从 [**Releases**](https://github.com/reedchan7/kimini/releases/latest) 下载
——Apple Silicon 选 `aarch64`，Intel 选 `x86_64`——把 **Kimini** 拖入
**应用程序** 即可。

> [!NOTE]
> 构建为 ad-hoc 签名，首次打开会被 Gatekeeper 拦截：
> 右键 → **打开**，或执行
> `xattr -dr com.apple.quarantine /Applications/Kimini.app`

```sh
# 首次启动——传入 `kimi web` 打印的 URL（含 #token=…），仅此一次：
open -na Kimini --args 'http://127.0.0.1:58627/#token=<daemon-token>'

# 之后——token 已持久化：
open -a Kimini
```

## 使用

| | |
|---|---|
| `⌘,` | 设置——宿主 UI 语言（English / 简体中文） |
| `⌘R` | 重新加载 |
| `⌘[` / `⌘]` | 后退 / 前进 |

启动 URL：命令行参数 → `$KIMINI_URL` → `http://127.0.0.1:58627/`。
语言：`$KIMINI_LANG`（`en` / `zh`）→ 已保存的偏好 → 系统区域设置。

## 从源码构建

```sh
make app            # → dist/Kimini.app   （Rust 1.85+）
make install-app    # → ~/Applications
make help           # 其余目标：run、lint、dmg、package-all、publish-release
```

## 说明

- 仅允许回环源（`127.0.0.1` / `::1` / `localhost`）；release 构建关闭 devtools。
- 尚未公证；已打包 `.app` 内的输入法未充分回归；目前仅支持 macOS。

---

[MIT](LICENSE) · 基于 [wry](https://github.com/tauri-apps/wry) /
[tao](https://github.com/tauri-apps/tao) /
[muda](https://github.com/tauri-apps/muda) 构建 ·
非官方项目，与 Moonshot AI 无隶属关系。
