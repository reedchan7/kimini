# AGENTS.md

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

## Rule 5 — Use the model only for judgment calls

Use me for: classification, drafting, summarization, extraction.
Do NOT use me for: routing, retries, deterministic transforms.
If code can answer, code answers.

## Rule 6 — Token budgets are not advisory

Per-task: 4,000 tokens. Per-session: 30,000 tokens.
If approaching budget, summarize and start fresh.
Surface the breach. Do not silently overrun.

## Rule 7 — Surface conflicts, don't average them

If two patterns contradict, pick one (more recent / more tested).
Explain why. Flag the other for cleanup.
Don't blend conflicting patterns.

## Rule 8 — Read before you write

Before adding code, read exports, immediate callers, shared utilities.
"Looks orthogonal" is dangerous. If unsure why code is structured a way, ask.

## Rule 9 — Tests verify intent, not just behavior

Tests must encode WHY behavior matters, not just WHAT it does.
A test that can't fail when business logic changes is wrong.

## Rule 10 — Checkpoint after every significant step

Summarize what was done, what's verified, what's left.
Don't continue from a state you can't describe back.
If you lose track, stop and restate.

## Rule 11 — Match the codebase's conventions, even if you disagree

Conformance > taste inside the codebase.
If you genuinely think a convention is harmful, surface it. Don't fork silently.

## Rule 12 — Fail loud

"Completed" is wrong if anything was skipped silently.
"Tests pass" is wrong if any were skipped.
Default to surfacing uncertainty, not hiding it.

## Project: Kimini native GUI and Web compatibility app

Rust Edition 2024 workspace that ships two independent macOS applications:

- `Kimini.app` / `kimini`: GPUI-native Kimi Code client.
- `Kimini Web.app` / `kimini-web`: WRY + tao + muda compatibility client.

Both applications share daemon discovery and session data. Native code consumes
the Kimi daemon's typed `/api/v1` REST and WebSocket contracts directly.

- Build native: `make build`; run: `make run`; package: `make app`.
- Build Web: `make build-web`; run: `make run-web`; package: `make app-web`.
- Build both bundles: `make apps`; both architectures and formats:
  `make package-all`.
- Release (preferred): project skill `.agents/skills/ship` (`/ship` / “ship”,
  default patch) via `scripts/ship.sh` → `make publish-release`. Read-only
  plan: `make ship-plan` or `bash scripts/ship.sh plan` (no fetch/write).
  Packaging dry-run only: `make publish-release PUBLISH_FLAGS=--dry-run`
  (builds `dist/`; not a plan-only mode). Claude Code: `.claude/skills/ship`
  → `.agents/skills/ship`.
- CI and release Actions remain manual (`workflow_dispatch`).
- Startup (Native and Web) discovers `~/.kimi-code/server/instances/*.json`
  (kimi-code 0.28+), falls back to legacy `server/lock`, reads `server.token`,
  health-probes candidates, and starts `kimi web --no-open` when none are
  healthy. The spawned server is process-group detached so Native and Web can
  share one origin without killing it when either app quits.
- Native modules stay directional: `daemon` and `api` feed `protocol` and the
  pure `model`; `native` renders projected state; `legacy_web` remains isolated.
- The native browser child view is on demand. It is for human preview and OAuth;
  future agent browsing belongs in an isolated Chromium broker.
- GPUI and gpui-component use the revisions recorded in `Cargo.lock`. Update
  them together and retain AccessKit coverage.
- Keep `wry 0.55` and `tao 0.34` paired for the Web app because their raw-window
  handle versions must match.
- Coverage policy: protocol and pure state logic stay at or above 90% line
  coverage. GUI/platform glue requires real-window and packaged-app scenarios.
- Resource claims cover the complete process family. Browser-off, browser-on,
  after-browser-close, and Web compatibility measurements are distinct.
- Product documentation stays English-only. Chinese UI text lives only in
  `src/i18n.rs`.
