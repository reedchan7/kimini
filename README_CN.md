<div align="center">

<img src="docs/brand/exports/app-icon-128.png" width="112" height="112" alt="Kimini 应用图标"/>

# Kimini

**专注于 Kimi Code 的原生桌面客户端，并保留独立的 Web 兼容应用。**

[English](README.md) · [简体中文](README_CN.md)

<a href="https://github.com/reedchan7/kimini/releases/latest"><img src="https://img.shields.io/badge/version-0.3.2-4A90D9?style=flat-square&logo=github" alt="版本 0.3.2"/></a>
<a href="#兼容性与发布数据"><img src="https://img.shields.io/badge/core%20coverage-97.03%25-brightgreen?style=flat-square&logo=rust" alt="核心覆盖率 97.03%"/></a>
<a href="#兼容性与发布数据"><img src="https://img.shields.io/badge/local%20tests-189%20passed-brightgreen?style=flat-square&logo=rust" alt="本地 189 项测试通过"/></a>
<img src="https://img.shields.io/badge/macOS-supported-black?style=flat-square&logo=apple&logoColor=white" alt="支持 macOS"/>
<img src="https://img.shields.io/badge/Linux-preview-FCC624?style=flat-square&logo=linux&logoColor=black" alt="Linux 预览版"/>
<img src="https://img.shields.io/badge/Windows-qualification-0078D4?style=flat-square&logo=windows" alt="Windows 目标机适配中"/>
<a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="MIT 许可证"/></a>

</div>

![Kimini 0.3 原生对话界面](docs/screenshots/kimini-native-overview.png)

Kimini 直接连接本地 [Kimi Code](https://github.com/MoonshotAI/kimi-code)
daemon，延续已有 session，并提供轻快的 GPUI 原生编码工作区。独立的
Kimini Web 则完整保留 daemon 提供的 Web 界面与行为。

## 选择适合你的版本

| | **Kimini** | **Kimini Web** |
|---|---|---|
| 适合场景 | 日常原生工作流 | 精确兼容 Web 界面 |
| 界面技术 | GPUI | 系统 WebView 中的 Kimi Code Web |
| 连接方式 | 类型化 REST + WebSocket | daemon 提供的 Web UI |
| 渲染技术 | Metal / Vulkan / DirectX | WKWebView / WebKitGTK / WebView2 |
| 浏览器开销 | 仅在人类预览时按需创建 | 系统 Web 进程族 |

两个应用可以同时安装；它们共用本地 daemon、认证与 session 数据。

## 平台进度

| 平台 | 架构 | 当前状态 |
|---|---|---|
| macOS 14+ | Apple Silicon、Intel | 已发布，带签名 Sparkle 更新通道 |
| Linux | x86_64、ARM64 | 便携预览版，面向 Debian 12 / Ubuntu 24.04 级运行环境 |
| Windows 10/11 | x86_64、ARM64 | 源码探针和打包链路就绪，等待目标主机完整验收 |

Linux 双架构归档可在本机通过 Docker Buildx 构建；两个应用的 x86_64 与
ARM64 归档都已在 Debian 12 / Ubuntu 24.04 环境中创建真实窗口。Windows
两个 MSVC target 均通过源码级交叉编译探针；release 仍固定在 Windows 上
完成，因为 GPUI 依赖原生工具链与 Windows SDK shader 编译器，ARM64 制品
还需 Windows ARM64 设备完成最终运行门禁。

## 为什么选择 Kimini

- **核心体验原生化。** 对话、输入框、session、设置、文件、任务、Skills、
  目标、终端与编码面板均由 GPUI 渲染。
- **延续现有 Kimi 工作流。** Kimini 自动发现或启动 `kimi server`，直接使用
  认证后的 `/api/v1` REST 与 WebSocket 契约。
- **Web 兼容始终可用。** Kimini Web 与 `kimi web` 共用界面、存储、session
  与认证。
- **同一代码库采用各平台原生渲染。** macOS 使用 Metal，Linux 使用 Vulkan，
  Windows 使用 DirectX；Web 版调用各操作系统自带的 Web 引擎。
- **浏览器资源按需发生。** 原生 Browser 面板开启后才创建预览。Linux
  Wayland 会把预览和 OAuth 链接交给系统浏览器；X11、macOS 与 Windows
  保留嵌入式子视图。
- **更新遵循各平台交付方式。** macOS 支持签名后的应用内替换；Linux 与
  Windows 的原生便携版会从更新入口打开最新 Release，避免提权自替换；
  Kimini Web 用户直接从 Releases 更新。

与 [Codex](https://openai.com/index/introducing-the-codex-app/) 这类通用 Agent
环境相比，Kimini 专注于已经在使用 Kimi Code 的开发者：一个 daemon、
一份历史，以及原生与 Web 两种桌面界面。这里不比较模型质量，也不发布
缺少同口径实验的跨产品性能结论。

## 快速开始

先安装 [Kimi Code](https://www.kimi.com/help/kimi-code/cli-getting-started)。
macOS 或 Linux：

```sh
curl -fsSL https://code.kimi.com/kimi-code/install.sh | bash
```

Windows 先安装 Git for Windows，再使用 PowerShell：

```powershell
irm https://code.kimi.com/kimi-code/install.ps1 | iex
```

打开新终端运行 `kimi`，首次启动时输入 `/login` 完成登录。

从 [Releases](https://github.com/reedchan7/kimini/releases/latest) 下载对应应用
与架构：

- macOS：`Kimini-<version>-macos-<arch>` 或 `Kimini-Web-...`（`.dmg` / `.zip`）
- Linux 预览版：`Kimini-<version>-linux-<arch>.tar.gz` 或 `Kimini-Web-...`
- Windows 目标机构建完成后：`Kimini-<version>-windows-<arch>.zip` 或
  `Kimini-Web-...`

Linux 需要 GTK 3、WebKitGTK 4.1、Vulkan 与桌面 portal。解压归档后运行
`bin/kimini` 或 `bin/kimini-web`；`share/` 中已包含 desktop entry 与图标。

Kimini 会读取平台用户目录下的 `.kimi-code/server/lock` 和 `server.token`，
探测 daemon 状态，并在需要时执行 `kimi server run`。凭据只进入请求头或
WebSocket 子协议，不会写入浏览器 URL。

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
| `⌘/Ctrl` + `Shift` + `E` | 文件 |
| `⌘/Ctrl` + `Shift` + `K` | Skills |
| `⌘/Ctrl` + `J` | 终端 |
| `⌘/Ctrl` + `Shift` + `T` | 任务 |

## 兼容性与发布数据

- 原生客户端使用 Kimi Code 自描述的 REST 与 WebSocket 协议，不解析
  CLI/TUI 输出。
- 原生终端优先使用 daemon 后端，同时保留本地 Rust PTY 回退路径。
- macOS aarch64 应用包含更新器时，原生版为 **17.1 MiB**、Web 版为
  **4.8 MiB**；Linux ARM64 便携归档分别为 **11.6 MiB** 与 **597 KiB**。
  归档与应用包采用不同统计口径。
- 协议与纯状态逻辑目前达到 **97.03% 行覆盖率**，本地原生/Web 套件包含
  **189 个自动化测试**。

原生应用仍处于早期发布阶段。Windows 打包应用行为、Linux 发行版广度、
CJK 输入、长 session 性能、流式无障碍、富媒体与完整交互式终端仍在持续
加固。CPU 与内存数据会在各平台完整进程族实验可以稳定复现后发布。

## 从源码构建

需要 Rust 1.96 或更高版本。

```sh
make run             # 在当前平台运行原生应用
make run-web         # 运行 Web 兼容应用
make package-all     # macOS：双应用、双架构、DMG + zip
make package-linux   # macOS/Linux：通过 Docker Buildx 生成双架构 tarball
```

在 Windows 的 Visual Studio 2022 Developer PowerShell 中运行：

```powershell
./scripts/package-windows.ps1 -App all -Arch all
```

Windows 构建机需要 Desktop C++ workload、x64/ARM64 MSVC 工具、Windows
10/11 SDK、对应 Rust target 与 WebView2。严格的本地发布协调器可以通过
`make publish-release-all` 汇总 macOS、Linux 与 Windows 制品，并拒绝缺失
任一平台或架构的发布矩阵。

更多细节见[平台研究](docs/research/2026-07-19-windows-linux-platform-support.md)、
[原生 GUI 规格](docs/native-gui-spec.md)和
[框架选型](docs/native-gui-framework-selection.md)。

---

Kimini 是社区独立项目。Kimi 与 Kimi Code 是 Moonshot AI 的产品；Kimini
与 Moonshot AI 无隶属关系。

[MIT](LICENSE) · [Issues](https://github.com/reedchan7/kimini/issues) · 基于
[GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui)、
[gpui-component](https://github.com/longbridge/gpui-component)、
[wry](https://github.com/tauri-apps/wry) 与
[tao](https://github.com/tauri-apps/tao) 构建；macOS 自动更新由
[Sparkle](https://sparkle-project.org/) 提供。
