# Direct Model API and Agent Harness Architecture for Kimini

Date: 2026-07-20

## Decision

Kimini Native can remove the **independent loopback Kimi server** as a mandatory
runtime dependency. It cannot replace that server with a single model API call
and retain Kimi Code behavior.

The recommended next-major direction is:

1. make the Native app own a Kimi agent-harness process over stdio/IPC;
2. start with the official `kimi acp` child-process contract;
3. keep the existing daemon backend during migration and for Kimini Web;
4. consider a versioned, bundled harness helper only after the official engine
   boundary is stable enough to package;
5. keep provider API calls inside the harness.

This removes the global port, bearer token, lock-file polling, and detached
daemon lifecycle from the Native launch path. It preserves the component that
plans turns, executes tools, applies permissions, persists sessions, compacts
context, and continues after tool results.

A raw direct-API mode is feasible as a small chat client. It is not a Kimi Code
replacement and should not become the default Kimini backend.

## Evidence boundary

Source snapshots used here:

- Kimini: the current repository source on 2026-07-20.
- Kimi Code: [`MoonshotAI/kimi-code@3086e47`](https://github.com/MoonshotAI/kimi-code/tree/3086e4703992fbbe7a41379405ee243713ad9ced).
- Codex: [`openai/codex@bcdc695`](https://github.com/openai/codex/tree/bcdc695877cc46d1475ccea295b6eda6fe4171cb).
- Claude Code and Kimi product/API facts: first-party documentation linked
  below, read on 2026-07-20.

No session transcript, credential, token, or private user data was read for this
research.

## 1. What Kimini starts today

The current Native launch flow is:

```text
Kimini GPUI
  -> discover lock + token, health-probe loopback
  -> start `kimi server run` when absent
  -> Kimi REST + WebSocket client
  -> kap-server
  -> Kimi agent engine
  -> model-provider API
```

[`native/bootstrap.rs`](../../src/native/bootstrap.rs) calls
`discover_connection`, then immediately loads sessions, workspaces, models,
auth, configuration, and the active-session projection. The discovery path
probes a recorded/default loopback origin and executes `kimi server run
--log-level error` when no healthy endpoint exists
([`daemon/discovery.rs`](../../src/daemon/discovery.rs),
[`daemon/process.rs`](../../src/daemon/process.rs), and
[`daemon/source.rs`](../../src/daemon/source.rs)).

The server is more than an HTTP proxy. Kimi Code's `kap-server`:

- takes the single-instance lock and manages bearer authentication;
- bootstraps `agent-core-v2` with file-backed storage for session metadata,
  records, blobs, and indexes;
- owns the model catalog and outbound model requests;
- exposes sessions, prompts, approvals, questions, tools, tasks, terminals,
  filesystem operations, files, snapshots, and live events.

The composition root and complete API tag list are visible in
[`packages/kap-server/src/start.ts` lines 160-247 and 351-415](https://github.com/MoonshotAI/kimi-code/blob/3086e4703992fbbe7a41379405ee243713ad9ced/packages/kap-server/src/start.ts#L160-L247).
The agent loop repeatedly requests model output and executes returned tool calls;
it is not a one-request relay
([`loopService.ts` lines 496-520 and 680-807](https://github.com/MoonshotAI/kimi-code/blob/3086e4703992fbbe7a41379405ee243713ad9ced/packages/agent-core-v2/src/agent/loop/loopService.ts#L496-L520)).

Therefore, “remove the daemon” contains two separable changes:

- remove its process/HTTP/WebSocket lifecycle;
- relocate or replace its agent engine.

The first is tractable. The second is the substantive product rewrite.

## 2. The standard harness GUI pattern

The premise that Codex and Claude Code GUIs call model APIs without a local
harness is not supported by their first-party sources.

| Product | Confirmed client-to-harness boundary | Confirmed model boundary |
| --- | --- | --- |
| Codex | `codex app-server` powers rich clients. External transports include stdio, Unix-socket WebSocket, and experimental TCP WebSocket. TUI and exec can run the same app-server in-process over typed bounded channels. | `codex-core` builds streamed Responses API requests with instructions, history, tools, reasoning controls, and provider authentication, then dispatches local tools. |
| Claude Code | The VS Code extension bundles its own CLI copy. Anthropic describes terminal, IDE, desktop, and web as surfaces over the same Claude Code engine. | The Agent SDK exposes the same built-in tools, agent loop, context management, permissions, sessions, hooks, MCP, and subagents as Claude Code; it uses an API key or supported provider credentials. |
| Kimi Code | The Web surface uses `kap-server`, while the TUI constructs `KimiHarness` in-process. Official IDE integration starts `kimi acp` as a child and uses JSON-RPC over stdin/stdout. | The harness selects a provider, streams model output, executes tools, returns tool results, maintains context, and persists the session. |

Codex's public app-server documentation explicitly calls it the interface for
rich clients and defines thread/turn/item lifecycle, streamed side effects, and
approvals
([`app-server/README.md` lines 1-3 and 64-81](https://github.com/openai/codex/blob/bcdc695877cc46d1475ccea295b6eda6fe4171cb/codex-rs/app-server/README.md#L1-L3)).
Its in-process client removes the process boundary while preserving app-server
semantics
([`app-server-client/README.md` lines 1-43](https://github.com/openai/codex/blob/bcdc695877cc46d1475ccea295b6eda6fe4171cb/codex-rs/app-server-client/README.md#L1-L43)).
`ModelClient` still builds tool-bearing streamed Responses requests inside the
core
([`core/src/client.rs` lines 824-908](https://github.com/openai/codex/blob/bcdc695877cc46d1475ccea295b6eda6fe4171cb/codex-rs/core/src/client.rs#L824-L908)).

Anthropic documents that the VS Code extension bundles the CLI
([Claude Code IDE integration](https://code.claude.com/docs/en/ide-integrations))
and that the Agent SDK is Claude Code's tools, loop, and context management as a
library
([Agent SDK overview](https://code.claude.com/docs/en/agent-sdk/overview)).
Anthropic's tool contract also states that client tools require an application
loop: receive `tool_use`, execute it, return `tool_result`, and continue
([How tool use works](https://platform.claude.com/docs/en/agents-and-tools/tool-use/how-tool-use-works)).

The reusable lesson is that a GUI may eliminate a **shared daemon** or a
network transport. A coding-agent GUI still needs a harness somewhere.

## 3. Kimi already supports non-daemon harness topologies

Kimi Code itself provides three relevant precedents.

### In-process CLI harness

The TUI creates `KimiHarness` directly
([`run-shell.ts` lines 35-79](https://github.com/MoonshotAI/kimi-code/blob/3086e4703992fbbe7a41379405ee243713ad9ced/apps/kimi-code/src/cli/run-shell.ts#L35-L79)).
The SDK constructs `KimiCore` and an in-memory RPC pair in the same process
([`sdk-rpc-client.ts` lines 48-90](https://github.com/MoonshotAI/kimi-code/blob/3086e4703992fbbe7a41379405ee243713ad9ced/packages/node-sdk/src/sdk-rpc-client.ts#L48-L90)).
This proves the daemon is not required by the agent engine.

### In-process v2 facade

The newer `Klient` contract deliberately supports HTTP, IPC, and memory
transports with the same facade. Its memory transport accepts a bootstrapped
engine scope; calls and events remain in-process
([`klient/README.md` lines 1-75](https://github.com/MoonshotAI/kimi-code/blob/3086e4703992fbbe7a41379405ee243713ad9ced/packages/klient/README.md#L1-L75) and
[`transports/memory/index.ts` lines 1-57](https://github.com/MoonshotAI/kimi-code/blob/3086e4703992fbbe7a41379405ee243713ad9ced/packages/klient/src/transports/memory/index.ts#L1-L57)).

This is the closest architectural match to “same engine, no local server.” It
cannot currently be linked directly into Rust: `agent-core-v2`, `klient`, and
the Node SDK are TypeScript packages marked `private`
([`agent-core-v2/package.json`](https://github.com/MoonshotAI/kimi-code/blob/3086e4703992fbbe7a41379405ee243713ad9ced/packages/agent-core-v2/package.json#L1-L6),
[`klient/package.json`](https://github.com/MoonshotAI/kimi-code/blob/3086e4703992fbbe7a41379405ee243713ad9ced/packages/klient/package.json#L1-L6), and
[`node-sdk/package.json`](https://github.com/MoonshotAI/kimi-code/blob/3086e4703992fbbe7a41379405ee243713ad9ced/packages/node-sdk/package.json#L1-L6)).
The repository is MIT-licensed, so a pinned helper build is legally possible,
while API stability and update ownership remain Kimini's responsibility.

### Official ACP child process

`kimi acp` is explicitly intended for editors, IDEs, and custom front ends. It
constructs the in-process harness, then exposes JSON-RPC over stdio
([`apps/kimi-code/src/cli/sub/acp.ts` lines 1-19 and 52-119](https://github.com/MoonshotAI/kimi-code/blob/3086e4703992fbbe7a41379405ee243713ad9ced/apps/kimi-code/src/cli/sub/acp.ts#L1-L19)).
The official IDE guide says the editor launches this child process and reuses
existing Kimi authentication
([Using Kimi Code CLI in IDEs](https://www.kimi.com/help/kimi-code/cli-ides)).

ACP currently advertises session list/load/resume, image prompts, MCP, model and
mode controls, streamed assistant/thinking/tool updates, and permission flows
([`acp-adapter/src/server.ts` lines 220-253](https://github.com/MoonshotAI/kimi-code/blob/3086e4703992fbbe7a41379405ee243713ad9ced/packages/acp-adapter/src/server.ts#L220-L253)).
It does not expose every current `/api/v1` management surface. Standalone
terminal tabs, arbitrary file transfer, complete global configuration, and the
full task/goal/cron management UI need a feature-by-feature gap assessment.

Moonshot's separate official Kimi Agent SDK reaches the same architectural
conclusion: its Go, Node.js, and Python clients are thin language-native layers
that keep Kimi CLI as the execution engine while exposing streaming, approvals,
tools, and sessions programmatically
([Kimi Agent SDK](https://github.com/MoonshotAI/kimi-agent-sdk)). There is no
official Rust client today, which makes the language-neutral ACP contract the
lower-risk Kimini boundary.

## 4. What the direct Kimi API provides

The official Kimi Code API offers OpenAI-compatible Chat Completions and an
Anthropic-compatible Messages endpoint. It currently documents `k3`,
`kimi-for-coding`, and `kimi-for-coding-highspeed`, with documented thinking
levels for K3
([Kimi Code overview](https://www.kimi.com/code/docs/en/)).

The official Kimi provider implementation sends system instructions, history,
tool schemas, streaming controls, thinking configuration, and parses streamed
tool-call deltas
([`kosong/providers/kimi.ts` lines 476-565](https://github.com/MoonshotAI/kimi-code/blob/3086e4703992fbbe7a41379405ee243713ad9ced/packages/kosong/src/providers/kimi.ts#L476-L565)).

That API supplies model inference. It does not run Kimini's local file, shell,
git, MCP, approval, task, or session code. Tool use is a loop owned by the
client harness: the model requests a tool, the client executes it, the client
returns the result, and the model continues.

Authentication also changes. The official third-party integration path tells
clients to create and configure a Kimi Code API key; it does not document reuse
of Kimi Code's consumer OAuth by an independent product
([Using in third-party coding agents](https://www.kimi.com/code/docs/en/third-party-tools/other-coding-agents)).
A standalone direct-API Kimini must therefore provide secure API-key setup and
storage, retain its real client identity, and avoid assuming the existing OAuth
session is portable.

## 5. Feasibility options

| Option | Removes shared daemon | Preserves official harness | Kimi Code parity potential | Main cost | Recommendation |
| --- | --- | --- | --- | --- | --- |
| Rust GUI calls model API directly | Yes | No | Low until Kimini rebuilds the engine | Agent loop, tools, permissions, sandbox, persistence, compaction, MCP, skills, tasks, recovery, auth | Reject as the default; acceptable only for a clearly labeled chat mode |
| Native app starts an installed or bundled `kimi acp` | Yes | Yes | High for core coding sessions; ACP gaps remain | Rust ACP client, event projection, child lifecycle, feature-gap closure | Best migration and default-runtime candidate |
| Native app ships a pinned TypeScript harness helper using engine + memory/IPC facade | Yes | Yes, at a pinned source revision | Highest controllable parity | Private/unstable package boundary, Node/native dependencies, signing, bundle/update ownership | Best possible end state if upstream boundaries stabilize |
| Port/rebuild Kimi agent-core in Rust | Yes | No; becomes Kimini's harness | Only through continuous reimplementation | Very large permanent maintenance and security surface | Defer; this changes the product into an independent agent |
| Keep current `kap-server` | No | Yes | Current baseline | Daemon lifecycle and version coupling | Keep for migration fallback and Kimini Web |

### Advantages of dropping the shared daemon from Native

- no loopback port, lock file, bearer token, CORS/origin policy, or health-poll
  startup path;
- app-owned lifecycle and clearer crash/restart reporting;
- an app-pinned helper can prevent client/server protocol drift;
- a private stdio channel has a smaller remotely reachable surface;
- Native can start only the harness needed for the active app lifecycle.

Potential memory, startup, and latency gains must be measured. Moving the same
Node engine into a child or embedded runtime does not inherently remove its
CPU or memory cost.

### Costs and regressions to plan for

- the daemon is currently the shared live source for Native and Web clients;
- app shutdown would stop in-process/background agents, goals, and scheduled
  work unless a separate continuation design exists;
- session-store concurrency and migration need explicit ownership when Web and
  Native use different engine processes;
- direct API mode requires API-key onboarding and loses the documented
  `kimi acp` reuse of existing login state;
- a bundled helper adds Node/native dependencies, cross-platform packaging,
  signing, update, and security-patch duties;
- ACP does not cover the entire current REST/WS surface;
- a Rust reimplementation must reproduce permission boundaries, sandboxing,
  cancellation, process cleanup, context compaction, provider quirks, tool-call
  repair, persistence, and recovery before it can claim parity.

## 6. Recommended target

```text
Kimini Native
  -> semantic backend boundary already projected into AppModel
  -> app-owned `kimi acp` child (stdio JSON-RPC)
  -> official Kimi harness
  -> Kimi/OpenAI/Anthropic-compatible provider API

Kimini Web
  -> existing `kimi server run`
  -> official Kimi Web + shared server contract
```

An app-owned child is materially different from today's daemon: it needs no
global TCP listener or discovery token, starts with Kimini, and terminates with
Kimini. It remains a process boundary, which provides useful crash and tool
isolation. Forcing the TypeScript engine into the Rust address space offers
little value relative to the integration and failure-containment cost.

If product requirements demand zero separately installed Kimi dependency, first
bundle a signed, version-pinned official Kimi single binary with the app and
continue to launch its public `acp` entry point. Build a dedicated
`kimini-harness` helper from a pinned source revision only if critical ACP gaps
cannot be closed upstream. Such a helper should use the engine's memory/IPC
facade and expose only the semantic operations Kimini consumes. Model API keys
should be read from OS secure storage and passed through the private channel,
never command-line arguments, environment dumps, URLs, logs, or session data.

## 7. Migration sequence and acceptance gates

### Phase A — backend seam and parity inventory

- Keep the existing Rust protocol/model/presentation reducer as the stable GUI
  boundary.
- Inventory every current operation: session/workspace lifecycle, prompt,
  steer/cancel, deltas, tool cards, approvals/questions, model/effort/mode,
  skills, tasks/goals, config/auth, files, terminal, reconnect, and recovery.
- Capture synthetic/isolated fixtures only. Never consume real user transcript
  content for contract tests.

### Phase B — ACP experimental backend

- Spawn the resolved `kimi` executable with `acp`; use JSON-RPC over piped
  stdin/stdout and reserve stderr for diagnostics.
- Implement the minimum ACP path: initialize, list/load/new/resume session,
  prompt, cancel, session updates, approval/question responses, model/mode
  options, and child shutdown.
- Project ACP events into the existing Kimini model instead of building a
  second UI state system.
- Keep daemon mode selectable as the known-good fallback.

### Phase C — close or consciously defer ACP gaps

- Use Kimini's existing local Rust PTY for user-owned terminal tabs.
- Map attachments, tasks, goals, skills, settings, session archive/fork/export,
  and background work one by one. Do not infer missing state from presentation
  widgets.
- If critical gaps require private engine APIs, stop extending ad hoc ACP
  metadata and build the pinned helper boundary instead.

### Phase D — default switch

Switch Native to the app-owned harness only after all of these hold:

- prompt/tool/approval/session behavior matches the existing isolated daemon
  scenarios;
- child crash, app restart, cancellation, orphan-process cleanup, and session
  resume behave deterministically;
- Native and Kimini Web cannot concurrently corrupt or overwrite shared state;
- macOS, Windows, and Linux packages include or locate the required harness
  consistently;
- API keys and OAuth artifacts never enter logs, command arguments, screenshots,
  or transcripts;
- complete process-family startup, memory, idle, streaming, and shutdown
  measurements beat or justify the current topology;
- Kimini Web remains functional through its existing server path.

### Phase E — optional independent Kimini harness

Take this step only if the product explicitly chooses independence from the
Kimi CLI runtime. At that point Kimini owns the agent engine, provider adapters,
tool sandbox, permissions, storage migration, and compatibility policy. That is
a separate product commitment and a valid major-version boundary.

## Final recommendation

Proceed with an ACP-backed Native spike and preserve the daemon backend as the
fallback. This can remove the awkward shared server lifecycle from normal
Native startup while retaining the official Kimi harness and login state.

Do not make raw direct model API calls the default architecture. They solve the
transport layer and leave the harder agent system unimplemented. Reconsider a
fully independent Rust harness only after the product intentionally accepts
loss of automatic Kimi Code behavioral parity and funds that engine as a
long-lived security-critical subsystem.
