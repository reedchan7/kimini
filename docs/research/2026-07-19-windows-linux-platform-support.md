# Windows and Linux Platform Support Research — 2026-07-19

This note evaluates `x86_64` and `aarch64` releases for both Kimini applications. It uses the revisions in `Cargo.lock`, first-party platform documentation, and a local cross-target probe. Product and dependency surfaces can change quickly; repeat the package and real-window acceptance gates before advertising a target as supported.

Labels used below:

- **Confirmed**: directly supported by pinned source, official documentation, or a reproduced local result.
- **Inference**: a proposed Kimini decision derived from those facts; it still needs implementation and target-machine acceptance.

## Executive decision

All four requested OS/architecture targets are technically viable for both applications. This change makes the shared source portable, adds local Linux dual-architecture builds, adds Windows-native dual-architecture packaging, and makes the release coordinator reject a partial portable matrix. Linux is ready for a portable preview; Windows remains gated on native target-host builds and runtime acceptance.

| Product | Windows x86_64 | Windows ARM64 | Linux x86_64 | Linux ARM64 |
|---|---:|---:|---:|---:|
| Kimini native (GPUI) | Feasible | Feasible | Feasible | Feasible |
| Kimini Web (tao + WRY) | Feasible | Feasible | Feasible | Feasible |
| Build from the current Mac | No supported release path | No supported release path | Docker Buildx | Docker Buildx / native ARM64 VM |
| Recommended release builder | Windows x64 host | Windows x64 host; ARM64 runtime host for acceptance | Native Linux x64 | Native Linux ARM64 |

The implemented workflow keeps platform builders explicit: Docker Buildx produces Linux archives locally, a Windows x64 host produces both MSVC architectures, and the Mac coordinates a strict release asset matrix. A single macOS linker path would still carry SDK, WebKitGTK, signing, and runtime gaps without removing the need for real target machines.

### Implementation result

- Shared home-directory, Kimi executable, shell, download, shortcut, URL-opening, and updater behavior is platform-aware.
- GPUI enables Linux X11 and Wayland. Kimini Web uses WRY's GTK builder on Linux; the native Browser pane falls back to the system browser on Wayland.
- Linux packaging emits reproducible `tar.gz` archives plus SHA-256 files for both applications and both architectures. A Docker Buildx wrapper makes this callable from macOS or Linux.
- Windows packaging runs in Visual Studio Developer PowerShell, embeds the application icon/version resource, and emits portable ZIPs plus SHA-256 files for x86_64 and ARM64.
- On Ubuntu 24.04 ARM64 and Debian 12 x86_64, all four release archives passed checksum validation and each extracted application created a real X11 window under an isolated display/session. The ARM64 archives measured 11.6 MiB native and 597 KiB Web; x86_64 measured 12.2 MiB and 624 KiB.
- Both Windows MSVC targets pass a local source-level cross probe. This is an early portability gate only; native packaging, launch, GPU, WebView2, and ARM64 runtime behavior remain Windows-host gates.

## Upstream capability at the pinned revisions

Kimini currently pins Zed/GPUI at [`9d7ab044`](https://github.com/zed-industries/zed/tree/9d7ab044366fb266cecb30b214aea8b7b94c032d) and gpui-component at [`03155566`](https://github.com/longbridge/gpui-component/tree/031555662e99a1b5a549990b47f246d475b8288a); see the repository [`Cargo.lock`](../../Cargo.lock).

### Native GPUI application

**Confirmed:** the pinned `gpui_platform::current_platform` selects `gpui_windows::WindowsPlatform` on Windows and `gpui_linux::current_platform` on Linux. Its feature map exposes Linux `wayland` and `x11` backends. [Pinned platform constructor](https://github.com/zed-industries/zed/blob/9d7ab044366fb266cecb30b214aea8b7b94c032d/crates/gpui_platform/src/gpui_platform.rs) · [Pinned feature/dependency map](https://github.com/zed-industries/zed/blob/9d7ab044366fb266cecb30b214aea8b7b94c032d/crates/gpui_platform/Cargo.toml)

**Confirmed:** Zed's release workflow at the same revision produces Windows and Linux releases for both architectures, including `Zed-x86_64.exe`, `Zed-aarch64.exe`, `zed-linux-x86_64.tar.gz`, and `zed-linux-aarch64.tar.gz`. The Windows jobs build both MSVC targets from an x64 Windows builder; Linux uses architecture-native builders. [Pinned Zed release workflow](https://github.com/zed-industries/zed/blob/9d7ab044366fb266cecb30b214aea8b7b94c032d/.github/workflows/release.yml) · [Pinned Windows bundler](https://github.com/zed-industries/zed/blob/9d7ab044366fb266cecb30b214aea8b7b94c032d/script/bundle-windows.ps1)

**Confirmed:** gpui-component describes its component set as cross-platform and contains Windows-specific native-menu code plus a drawn-menu fallback for Linux. [Pinned gpui-component README](https://github.com/longbridge/gpui-component/blob/031555662e99a1b5a549990b47f246d475b8288a/README.md) · [Pinned platform dependencies](https://github.com/longbridge/gpui-component/blob/031555662e99a1b5a549990b47f246d475b8288a/crates/ui/Cargo.toml) · [Pinned native-menu dispatch](https://github.com/longbridge/gpui-component/blob/031555662e99a1b5a549990b47f246d475b8288a/crates/ui/src/native_menu/mod.rs)

**Conclusion:** GPUI itself does not block these targets. Kimini's platform assumptions do.

### Web compatibility application and embedded browser

**Confirmed:** Kimini resolves to WRY `0.55.1` and tao `0.34.8`. WRY uses WebView2 on Windows and WebKitGTK 4.1/GTK 3 on Linux. Its raw-window path supports Windows and Linux X11; its GTK path is the documented route for supporting Linux X11 and Wayland. Child webviews created with `build_as_child` support Windows and Linux X11 only. [WRY 0.55.1 source documentation](https://docs.rs/crate/wry/0.55.1/source/README.md) · [tao 0.34.8 source documentation](https://docs.rs/crate/tao/0.34.8/source/README.md) · [WebKitGTK 4.1 API](https://webkitgtk.org/reference/webkit2gtk/stable/)

That produces two distinct Linux outcomes:

- **Kimini Web:** feasible on X11 and Wayland after using WRY's GTK builder on Linux.
- **Native embedded browser:** feasible as a GPUI child on X11 and Windows. There is no documented WRY raw-window child path for Linux Wayland.

**Inference:** keep the native application fully supported on Linux Wayland, but open browser/OAuth URLs externally there. Retain the embedded child on X11. Creating a second GTK event loop or a separate GTK window solely to preserve the child pane would add disproportionate complexity. Kimini Web remains the exact Web compatibility surface on both Linux compositors.

### Kimi Code compatibility

**Confirmed:** the current Kimi Code CLI supports macOS, Linux, and Windows PowerShell, stores its data under `~/.kimi-code` on all three systems, and implements `kimi server` using launchd, user systemd, or Windows Scheduled Tasks. Windows requires Git for Windows because the CLI uses Git Bash. [Official installation guide](https://www.kimi.com/help/kimi-code/cli-getting-started) · [Official data locations](https://www.kimi.com/code/docs/en/kimi-code-cli/configuration/data-locations.html) · [Official `kimi server` reference](https://www.kimi.com/code/docs/en/kimi-code-cli/reference/kimi-command.html)

**Conclusion:** the daemon dependency is available on every requested target; Kimini must follow its Windows executable, home-directory, and shell conventions.

## Pre-implementation gaps resolved in this change

These findings came from the pre-change repository and drove the implementation.

1. **Updater linkage:** [`src/updater.rs`](../../src/updater.rs) is Sparkle/Objective-C code and [`src/lib.rs`](../../src/lib.rs) exports it only on macOS, while native [`Shell`](../../src/native/app.rs) imports and owns `Updater` unconditionally. Windows and Linux native builds therefore cannot compile once their system dependencies are present.
2. **Linux compositor features:** [`Cargo.toml`](../../Cargo.toml) enables `font-kit` and `runtime_shaders` on `gpui_platform`, but not its `x11` or `wayland` features. The pinned Linux backend explicitly requires at least one of them for a windowed application.
3. **Linux browser construction:** both [`native/browser/controller.rs`](../../src/native/browser/controller.rs) and [`legacy_web/app.rs`](../../src/legacy_web/app.rs) use raw-window WRY builders. The documented Linux Wayland path needs the GTK builder; the native child needs the external-browser fallback described above.
4. **Windows process and data discovery:** [`daemon/source.rs`](../../src/daemon/source.rs) assumes `HOME`; [`daemon/process.rs`](../../src/daemon/process.rs) probes extensionless `kimi` paths; and the local terminal falls back to `/bin/zsh`. Explorer-launched Windows apps need `USERPROFILE`/known-folder fallback, `kimi.exe`/launcher resolution, and `KIMINI_SHELL_PATH`/`KIMI_SHELL_PATH` or PowerShell selection.
5. **Keyboard semantics:** the native bindings use `cmd-*`. GPUI documents `secondary-*` as Command on macOS and Control on Windows/Linux, which is the portable spelling. [Zed key-binding reference](https://zed.dev/docs/key-bindings)
6. **External URL launch:** Kimini Web currently invokes `cmd /c start` on Windows. A shell-parsed URL is an avoidable command boundary; use the Windows shell API or an existing GPUI/tao URL opener.
7. **Desktop integration:** Web menus/settings are currently macOS-only. This does not block the Web shell, but the Windows and Linux release acceptance list must decide whether toolbar/OS-menu parity is required.
8. **Release tooling:** all packaging, signing, updater feeds, CI matrices, and local publication scripts currently describe macOS only.

## Can the Mac build these targets directly?

### Windows

**Confirmed:** Rust classifies `x86_64-pc-windows-msvc` and `aarch64-pc-windows-msvc` as Tier 1 with host tools. Cross-architecture builds from one Windows host are supported when the matching Visual Studio components are installed; cross-compilation from a non-Windows host to MSVC is explicitly unsupported. [Rust Windows MSVC target support](https://doc.rust-lang.org/stable/rustc/platform-support/windows-msvc.html)

The pinned GPUI Windows release build also locates `fxc.exe` through the Windows SDK to compile HLSL. Zed's official setup requires Visual Studio C++ Build Tools and a Windows 10/11 SDK. [Pinned GPUI Windows shader build](https://github.com/zed-industries/zed/blob/9d7ab044366fb266cecb30b214aea8b7b94c032d/crates/gpui_windows/build.rs) · [Official Zed Windows build guide](https://zed.dev/docs/development/windows)

**Conclusion:** the current Mac cannot produce a supported Windows release directly. Use one x64 Windows builder for both binaries/architectures, then run the ARM64 packages on Windows ARM64 before release.

### Linux

**Confirmed:** Rust's ARM64 GNU/Linux target is Tier 1 and can be cross-compiled from any host when the matching C compiler is available. Native-library crates still require target libraries and correct build-script configuration. [Rust ARM64 Linux target support](https://doc.rust-lang.org/rustc/platform-support/aarch64-unknown-linux-gnu.html) · [Cargo build-script and `-sys` guidance](https://doc.rust-lang.org/cargo/reference/build-scripts.html)

**Local result:** on this Apple Silicon Mac, both current Kimini features reached GTK system crates for `aarch64-unknown-linux-gnu`, then stopped in `glib-sys`/`gobject-sys` because target `pkg-config`, sysroot, and GTK libraries are not configured. This confirms that adding a Rust target alone is insufficient.

**Conclusion:** a hermetic Linux sysroot could make Mac-hosted cross-compilation possible, but it would still not provide compositor, WebKitGTK, install, or updater acceptance. Native Linux builders are the smaller and more reliable release path. The official Zed pipeline reaches the same conclusion by using native x86_64 and ARM64 Linux builders. [Official Zed Linux build guide](https://zed.dev/docs/development/linux)

## Build and runtime dependencies

### Windows 10/11

- Rust targets: `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`.
- Visual Studio 2022/Build Tools with Desktop C++, x64 and ARM64 MSVC libraries, plus the Windows 10/11 SDK. Release GPUI builds need the SDK shader compiler.
- DirectX 11-capable GPU for the current GPUI Windows renderer. [Zed Windows runtime requirements](https://zed.dev/docs/windows)
- WebView2 Evergreen Runtime for Kimini Web and the native embedded browser. Microsoft recommends detecting/installing it during setup; Windows 11 includes it, while some Windows 10 systems may not. Architecture-matched x64 and ARM64 installers exist. [WebView2 distribution](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)
- Kimi Code CLI and Git for Windows.

### Linux

- Rust target matching the host architecture, C/C++ toolchain, `pkg-config`, and a glibc baseline selected by the oldest supported build distribution.
- GPUI runtime/build libraries: fontconfig, GLib, Wayland, X11/XCB, xkbcommon, Vulkan loader/driver, and XDG desktop portal. The pinned Zed dependency script is the authoritative starting list. [Pinned Linux dependency script](https://github.com/zed-industries/zed/blob/9d7ab044366fb266cecb30b214aea8b7b94c032d/script/linux)
- WRY: GTK 3 and WebKitGTK 4.1 (`libwebkit2gtk-4.1-dev` on Debian/Ubuntu; `webkit2gtk4.1-devel` plus GTK 3 development files on Fedora). [WRY 0.55.1 Linux dependencies](https://docs.rs/crate/wry/0.55.1/source/README.md)
- Kimi Code CLI. Linux service installation uses user systemd when the user chooses it.

**Inference:** initially qualify one conservative Linux baseline (for example Ubuntu 22.04+/Debian 12 class systems) and test additional distributions. Do not claim universal Linux compatibility from a single successful build; WebKitGTK and graphics-driver versions are runtime inputs.

## Packaging and update strategy

The first cross-platform release format is deliberately portable: per-architecture Linux `tar.gz` and Windows ZIP archives with SHA-256 files. This gives the project a local, auditable build path before it acquires platform signing identities. Portable builds route update requests to the latest GitHub Release; macOS retains Sparkle.

The managed-install formats below remain the production evolution once their signing and clean-machine gates are funded.

### Windows: MSIX bundle + App Installer

**Recommendation:** create one signed `.msixbundle` per application containing x64 and ARM64 packages, plus one stable `.appinstaller` file per application. Windows downloads the applicable architecture from a bundle, and App Installer supports launch/background updates for packages hosted outside the Store. [MSIX bundles](https://learn.microsoft.com/en-us/windows/msix/packaging-tool/bundle-msix-packages) · [App Installer automatic updates](https://learn.microsoft.com/en-us/windows/msix/app-installer/auto-update-and-repair--overview)

Use Windows SDK `MakeAppx.exe`/`SignTool.exe`; map Cargo `0.3.2` to MSIX's four-part numeric `0.3.2.0`. Keep separate package identities for Kimini and Kimini Web. MSIX signing is mandatory, the certificate must be trusted, and publisher identity must remain stable across updates. [MakeAppx](https://learn.microsoft.com/en-us/windows/msix/package/create-app-package-with-makeappx-tool) · [MSIX signing](https://learn.microsoft.com/en-us/windows/msix/package/signing-package-overview)

**Release blocker:** obtain a production-trusted Windows signing identity. Self-signed MSIX is suitable only for development because every user must trust the certificate manually.

### Linux: AppImage, with an explicit preview gate for ARM64

**Recommendation:** use architecture-specific AppImages and `.zsync` metadata. AppImage supports embedded update information and invoking bundled `AppImageUpdate` from an “Update…” action; its guidance calls for explicit consent or opt-in before downloads. [AppImage updates](https://docs.appimage.org/packaging-guide/optional/updates.html) · [AppImage format](https://docs.appimage.org/reference/architecture.html)

Two caveats must remain visible:

- WRY relies on the system WebKitGTK stack, so AppImage does not remove the need for distribution/runtime testing.
- Official AppImage documentation says common prebuilt packaging tooling is strongest on x86_64 while the tools should support ARM; its Open Build Service example includes `aarch64`. Treat ARM64 AppImage production as a measured gate, not an assumption. [Native AppImage packaging](https://docs.appimage.org/packaging-guide/from-source/native-binaries.html) · [AppImage ARM build example](https://docs.appimage.org/packaging-guide/hosted-services/opensuse-build-service.html)

If the ARM64 AppImage gate fails, publish an architecture-specific `.tar.gz` preview with checksums and manual updates. Avoid silently presenting that fallback as equivalent to the signed, self-updating macOS and Windows channels.

### Updater ownership by package type

| Package | Update owner | Kimini behavior |
|---|---|---|
| macOS app | Sparkle | Existing signed appcast flow |
| Windows MSIX | Windows App Installer | Automatic OS-managed updates; manual action opens the stable `.appinstaller` URI |
| Linux AppImage | AppImageUpdate | User-approved in-app update using embedded update information |
| Linux distro/Flatpak package, if added later | Package manager | Disable self-replacement and open package-manager instructions |
| Bare development binary/tar fallback | None | Open the latest GitHub release |

This keeps update mechanics platform-owned and avoids inventing a second privileged self-replacement protocol.

## Portable release matrix implemented now

The strict local coordinator expects the following portable assets in addition to the existing macOS artifacts:

| OS | Assets | Architectures | Files including checksums |
|---|---|---|---:|
| Windows | `Kimini-<version>-windows-<arch>.zip` | x86_64 + ARM64 | 4 |
| Windows | `Kimini-Web-<version>-windows-<arch>.zip` | x86_64 + ARM64 | 4 |
| Linux | `Kimini-<version>-linux-<arch>.tar.gz` | x86_64 + ARM64 | 4 |
| Linux | `Kimini-Web-<version>-linux-<arch>.tar.gz` | x86_64 + ARM64 | 4 |

## Future managed-install matrix

The smallest signed, self-updating production matrix would be:

| OS | Asset | Architectures represented | Count |
|---|---|---|---:|
| Windows | `Kimini-<version>-windows.msixbundle` | x64 + ARM64 | 1 |
| Windows | `Kimini-Web-<version>-windows.msixbundle` | x64 + ARM64 | 1 |
| Windows | stable `.appinstaller` files | one per app | 2 |
| Linux | `Kimini-<version>-linux-<arch>.AppImage` | one each for x86_64/ARM64 | 2 |
| Linux | `Kimini-Web-<version>-linux-<arch>.AppImage` | one each for x86_64/ARM64 | 2 |
| Linux | matching `.zsync` files | one per AppImage | 4 |

`.deb`, `.rpm`, Flatpak, and package-manager repositories should be added only when a measured distribution need appears. Each extra format creates another install, uninstall, dependency, and update contract.

## Release-flow shape

**Inference:** retain a platform packager per OS and let publication aggregate completed assets:

1. macOS builder produces the existing two apps, two architectures, DMG/ZIP, and Sparkle feeds.
2. Windows x64 builder compiles x64 + ARM64, creates two MSIX bundles and two App Installer files, and signs them.
3. Linux x86_64 and ARM64 builders each produce both AppImages and `.zsync` files.
4. A final publication step rejects a partial matrix, checks version/architecture metadata and signatures, then uploads the fixed asset set to one GitHub Release.

The current Mac can remain the local coordinator, but remote/native builders must return immutable artifacts and checksums. Keep the existing manual `workflow_dispatch` path as an optional reproducible fallback; do not make GitHub Actions a prerequisite for local releases.

## Acceptance gates before changing README support claims

For every OS/architecture/product cell:

- compile and test the exact release target;
- launch a real packaged window and exercise IME, clipboard, file dialogs, accessibility, scaling, suspend/resume, and GPU fallback;
- discover/start the real Kimi daemon, authenticate, load sessions, send a prompt, and open a terminal;
- exercise Kimi Web navigation and persistent storage;
- exercise the native browser behavior appropriate to Windows, Linux X11, and Linux Wayland;
- install version N, update to N+1 through the shipped package channel, relaunch, and confirm user/session data survives;
- confirm install, uninstall, signature/trust behavior, and clean-machine runtime dependencies.

Until those gates pass, README wording should say **experimental Windows/Linux builds** or keep the platform in the roadmap. The upstream libraries establish feasibility; they do not establish Kimini release quality.
