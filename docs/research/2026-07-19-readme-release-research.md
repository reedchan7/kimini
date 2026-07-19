# README Release Research — 2026-07-19

This note is intentionally limited to primary sources and claims suitable for a concise public README. Product surfaces change quickly; re-check dated competitor details before later releases.

## 1. GitHub's README guidance

Source: [GitHub Docs — About the repository README file](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-readmes)

GitHub says a README is often the first item a visitor sees and should explain what the project does, why it is useful, how to get started, where to get help, and who maintains it. GitHub also recommends relative links for repository files and keeping longer material outside the README.

Recommended Kimini order:

1. One-sentence product promise, language switch, status/version badges.
2. One strong native-app screenshot.
3. A short **Choose your app** table: Kimini (GPUI) vs Kimini Web (WRY/WebKit).
4. Three to five user-facing advantages.
5. Download or build quick start.
6. Kimi Code compatibility and the local-daemon relationship.
7. Small, dated measurements table with methodology linked elsewhere.
8. Current limitations, support, contributing, and license.

This puts the decision and first successful launch before architecture details.

## 2. Kimi Code and Kimi Web: official facts

Sources:

- [MoonshotAI/kimi-code](https://github.com/MoonshotAI/kimi-code)
- [Kimi Code overview](https://www.kimi.com/code/docs/en/)
- [`kimi` command reference](https://www.kimi.com/code/docs/en/kimi-code-cli/reference/kimi-command.html)

Safe facts:

- Kimi Code CLI is a terminal coding agent that can read and edit files, run shell commands, search files, fetch web pages, and adapt its next action from feedback.
- The official installation entry points include the install script and Homebrew; first use supports Kimi Code OAuth or a Moonshot AI Open Platform API key.
- `kimi web` is the official browser-based graphical session. It is an alias for `kimi server run --open`: it starts or reuses the local Kimi server and opens its Web UI.
- The local server exposes REST and WebSocket services; its OpenAPI and AsyncAPI documents are available at `/openapi.json` and `/asyncapi.json` while it is running.
- Kimi Code supports official clients and third-party development tools. Its public model API offers OpenAI- and Anthropic-compatible endpoints; that API compatibility is separate from compatibility with the local daemon API.

Kimini wording should explicitly say **community-built** or **unofficial**. It may say it uses the same local daemon, sessions, and authentication as Kimi Code only when the release's source and packaged-app run substantiate those paths. Avoid “100% compatible”; name the exact supported Kimi Code version or contract instead.

## 3. OpenAI Codex: current official positioning

Sources:

- [OpenAI — Introducing the Codex app](https://openai.com/index/introducing-the-codex-app/)
- [openai/codex](https://github.com/openai/codex)
- [OpenAI Help — Moving to the new ChatGPT desktop app](https://help.openai.com/en/articles/20001276/)

Current facts:

- Codex is available through a local CLI, IDE integration, desktop experience, and cloud/Web surface.
- Its desktop experience supports parallel agents, project/thread organization, diff review, editor handoff, worktree isolation, skills, and shared CLI/IDE session history and configuration.
- OpenAI's current desktop transition combines Chat, Work, and Codex in the ChatGPT desktop app on macOS and Windows.

A defensible comparison is about workflow fit:

- **Kimini:** a focused macOS client for existing Kimi Code users, with a GPUI-native interface and a separate Web compatibility app over the Kimi daemon.
- **Codex:** a broader OpenAI agent environment spanning desktop, CLI, IDE, and cloud, with first-party multi-agent and worktree workflows.

Do not claim Codex lacks a native GUI, Web access, session continuity, worktrees, or parallel agents. Do not compare model quality, task success, security, startup speed, memory, binary size, or cost without an apples-to-apples, dated benchmark and clearly stated process boundaries.

## 4. Claims suitable for the release README

Recommended copy:

- “A native macOS GUI for Kimi Code, with a separate Web compatibility app.”
- “Keep working with your local Kimi Code daemon and sessions from a focused desktop interface.”
- “Choose GPUI-native Kimini for the desktop experience, or Kimini Web when exact browser UI compatibility matters.”
- “Community-built and independent; Kimi and Kimi Code are products of Moonshot AI.”

Measurements may include signed app-bundle size, cold/warm time to an interactive window, and complete process-family memory for native browser-off, native browser-on, after browser close, and Web compatibility. Each value needs the Kimini/Kimi versions, hardware, macOS version, sample count, and measurement date. Prefer absolute results over “faster than Codex” unless both products were measured under the same protocol.
