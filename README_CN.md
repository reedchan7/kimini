<div align="center">

<img src="docs/brand/exports/app-icon-128.png" width="112" height="112" alt="Kimini 应用图标"/>

# Kimini

**Kimi Code 原生 macOS GUI，Web 体验始终触手可及。**

[English](README.md) · [简体中文](README_CN.md)

<a href="https://github.com/reedchan7/kimini/releases/latest"><img src="https://img.shields.io/badge/version-0.3.1-4A90D9?style=flat-square&logo=github" alt="版本 0.3.1"/></a>
<a href="#兼容性与发布数据"><img src="https://img.shields.io/badge/core%20coverage-97.03%25-brightgreen?style=flat-square&logo=rust" alt="核心覆盖率 97.03%"/></a>
<a href="#兼容性与发布数据"><img src="https://img.shields.io/badge/local%20tests-188%20passed-brightgreen?style=flat-square&logo=rust" alt="本地 188 项测试通过"/></a>
<img src="https://img.shields.io/badge/platform-macOS%2014%2B-black?style=flat-square&logo=apple&logoColor=white" alt="macOS 14+"/>
<a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="MIT 许可证"/></a>

</div>

![Kimini 0.3 原生对话界面](docs/screenshots/kimini-native-overview.png)

Kimini 0.3 把本地 [Kimi Code](https://github.com/MoonshotAI/kimi-code)
工作流带进专注、轻快的 GPUI 桌面应用。它直接连接 Kimi daemon，延续已有
session，同时用独立的 Kimini Web 保留浏览器工作流。

## 选择适合你的版本

| | **Kimini** | **Kimini Web** |
|---|---|---|
| 适合场景 | 日常原生工作流 | 浏览器界面兼容 |
| 界面技术 | GPUI + Metal | 系统 WKWebView 中的 Kimi Code Web |
| 连接方式 | 类型化 REST + WebSocket | daemon 提供的 Web UI |
| 渲染进程 | 无常驻浏览器渲染器 | WebKit 进程族 |
| 应用包 | **17.1 MiB** | **4.8 MiB** |

两个版本可以同时安装、并行运行，共用本地 daemon 与 session 数据。

## 为什么选择 Kimini

- **核心体验原生化。** 对话、输入框、session、设置与编码面板由 GPUI 和
  Metal 渲染。
- **延续现有 Kimi 工作流。** Kimini 自动发现或启动本地 daemon，并使用其
  认证后的 `/api/v1` REST 与 WebSocket 契约。
- **Web 兼容随时可用。** 仍需浏览器行为的功能可以直接交给独立的
  Kimini Web。
- **编码能力集中呈现。** 文件、搜索、预览、Git 状态、任务、Skills、目标、
  side chat、审批、prompt 队列与终端都在一个桌面工作区中。
- **浏览器开销按需发生。** 原生应用只在打开 Browser 时创建预览用
  WKWebView，关闭面板后销毁视图。
- **内置签名更新。** 两个应用每天自动检查并可在后台安装 Ed25519 签名
  的更新，也可以随时手动检查。

与 [Codex](https://openai.com/index/introducing-the-codex-app/) 这类覆盖桌面、
CLI、IDE 与云端的通用 Agent 环境相比，Kimini 的优势在于对 Kimi Code
工作流的专注：一个 daemon、一份 session 历史，以及原生与 Web 两种桌面
界面。这里不比较模型质量，也不宣称缺少同口径实验的跨产品性能胜负。

## 快速开始

需要 macOS 14+ 与 Kimi Code。

```sh
curl -fsSL https://code.kimi.com/kimi-code/install.sh | bash
kimi login
```

从 [Releases](https://github.com/reedchan7/kimini/releases/latest) 下载对应架构：

- `Kimini-<version>-macos-<arch>` — 原生 GPUI 应用
- `Kimini-Web-<version>-macos-<arch>` — Web 兼容应用

当前构建使用 ad-hoc 签名。首次启动若被 macOS 拦截，请右键应用并选择
**打开**。

带更新器的构建使用 Sparkle 完成原子替换与重启，默认每天检查一次；也可
随时点击 **检查更新…**。

Kimini 会读取 `~/.kimi-code/server/lock` 与 `server.token`，探测 daemon
状态，并在需要时执行 `kimi server run`。凭据只进入请求头或 WebSocket
子协议，不会写入浏览器 URL。

## 原生体验

0.3 已支持 session 创建、搜索、重命名、归档/恢复、fork、compact、undo、
附件、流式输出、thinking 与工具轨迹、审批与问题、运行模式、文件、任务、
Skills、目标、side chat、终端、多语言、主题、认证与键盘优先导航。

<details>
<summary><strong>设置、外观、语言、账户与 Agent 默认值</strong></summary>

![Kimini 原生设置](docs/screenshots/kimini-native-settings.png)

</details>

| 快捷键 | 面板 |
|---|---|
| `⌘⇧E` | 文件 |
| `⌘⇧K` | Skills |
| `⌘J` | 终端 |
| `⌘⇧T` | 任务 |

## 兼容性与发布数据

- 原生客户端使用 Kimi Code 自描述的 REST 与 WebSocket 协议，不解析
  CLI/TUI 输出。
- Kimini Web 始终作为独立应用保留，与 `kimi web` 共用 session 和认证。
- 原生终端优先使用 daemon 后端；打包后的 daemon 无法加载 PTY 模块时，
  可回退到本地 Rust PTY。
- 两个 arm64 应用可以一起构建和运行；嵌入更新器后，当前应用包大小为
  原生 17.1 MiB、Web 4.8 MiB。
- 协议与纯状态逻辑目前达到 **97.03% 行覆盖率**，本地原生/Web 发布套件
  包含 **188 个自动化测试**。

原生应用仍处于早期发布阶段。打包场景下的 CJK 输入、长 session 性能、
流式无障碍、富媒体与完整交互式终端能力仍在持续加固。完整进程族的同口径
性能实验结束后再发布 CPU 与内存数据。

## 从源码构建

```sh
make run          # 原生应用
make run-web      # Web 兼容应用
make apps         # 在 dist/ 生成两个 .app
make package-all  # 双架构、DMG + zip
```

首次打包会下载固定版本的 Sparkle。执行本地发布时，还会签名四份按应用与
架构区分的更新源，并与应用压缩包一起上传。

更深入的设计与性能方法见[原生 GUI 规格](docs/native-gui-spec.md)和
[框架选型](docs/native-gui-framework-selection.md)。

---

Kimini 是社区独立项目。Kimi 与 Kimi Code 是 Moonshot AI 的产品；Kimini
与 Moonshot AI 无隶属关系。

[MIT](LICENSE) · [Issues](https://github.com/reedchan7/kimini/issues) · 基于
[GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui)、
[gpui-component](https://github.com/longbridge/gpui-component)、
[wry](https://github.com/tauri-apps/wry) 与
[tao](https://github.com/tauri-apps/tao) 构建，自动更新由
[Sparkle](https://sparkle-project.org/) 提供。
