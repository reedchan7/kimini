---
name: ship
description: >
  Full Kimini release pipeline via scripts/ship.sh: sync main, commit with
  git-commit, idempotent version bump, push, package dual-arch artifacts, and
  publish GitHub Release (tag owned by publish-release after packaging). Use
  when the user says ship, /ship, "发布", ship patch|minor|major, or ship
  <VERSION>. Default bump is patch. Dry-run is read-only.
---

# ship — Kimini release pipeline

**Canonical path:** `.agents/skills/ship/` (Claude: `.claude/skills/ship` symlink).

Deterministic work lives in `scripts/ship.sh`. This skill only orchestrates:
args → (optional read-only plan) → commits via **git-commit** → push → publish.
**Do not** create or push tags here; `scripts/publish-release.sh` tags **after**
packaging/signing succeed.

## Invocation

| Input | Version |
|-------|---------|
| `ship` / `/ship` / `发布` | `--bump default` (patch only if needed) |
| `ship patch` \| `minor` \| `major` | Force mode (idempotent vs latest published) |
| `ship 0.4.0` / `ship v0.4.0` | Exact semver |
| `--dry-run` | **Read-only plan only** (no stash/commit/push/tag/publish) |
| `--draft` | GitHub Release as draft |
| `--portable` | `publish-release-all` (Windows assets must be **pre-staged** in `dist/`) |

Invoking ship authorizes push + Release publish unless `--dry-run`. Never
force-push. Never `git commit --no-verify`. Branch must be **`main`** (no override).

## Hard requirements

Stop if any fail:

1. `uname -s` is Darwin for real publish (plan may run without packaging tools)
2. Package name `kimini` (via `scripts/ship.sh plan`)
3. Branch `main`
4. `gh` auth + `origin` → `reedchan7/kimini` (publish step)
5. No merge/rebase/cherry-pick in progress

## Pipeline

### 0. Parse

Extract: `BUMP` (`default`|`patch`|`minor`|`major`|`X.Y.Z`), `DRY_RUN`, `DRAFT`,
`PORTABLE`. Strip leading `v` from exact versions.

### 1. Dry-run (read-only) — may exit here

```bash
bash scripts/ship.sh plan --bump "$BUMP" ${PORTABLE:+--portable} ${DRAFT:+--draft}
```

If `DRY_RUN=1`: print the plan and **stop**. No pull, stash, commit, version
edit, push, tag, or publish.

### 2. Lock + gates + fetch

```bash
bash scripts/ship.sh lock
# always unlock on exit (success or failure)
trap 'bash scripts/ship.sh unlock' EXIT
bash scripts/ship.sh require-main
bash scripts/ship.sh require-canonical-remote   # BEFORE any push
bash scripts/ship.sh require-no-in-progress
git fetch origin main --tags
# re-plan after fetch with fresh origin/main
bash scripts/ship.sh plan --bump "$BUMP"
```

If behind `origin/main`: first commit local work (step 3) so the tree is clean,
then `git rebase origin/main`. **No auto-stash.** Stop on conflicts.

If push is later rejected: unlock, stop, tell user to re-run full ship after
sync — **do not** rebase-and-retry push only.

### 3. Commit pending work (always load git-commit first)

Load `git-commit` skill before **any** commit (feature or version).

If dirty (and not only an in-progress version apply you own):

1. Follow git-commit end-to-end
2. Commit intentional project changes only; do **not** `git add -A` blindly if
   unexpected new paths appear mid-run — stop and report
3. Split themes when git-commit requires it

If only version files will change, skip to step 4.

### 4. Version

```bash
bash scripts/ship.sh plan --bump "$BUMP"   # re-read after commits/rebase
```

- `action=nothing` → stop: nothing to ship  
- `action=keep` → use `target_version` as-is  
- else:

  ```bash
  bash scripts/ship.sh apply-version "$target_version"
  ```

  Then commit with **git-commit**:

  ```
  chore: bump version to X.Y.Z
  ```

Set `RELEASE_VERSION` / `EXPECTED_SHA=$(git rev-parse HEAD)`.

### 5. Preflight

```bash
bash scripts/ship.sh preflight   # clean tree + make check + make test
```

On failure: fix blockers only or stop. Do not refactor unrelated code.

### 6. Push branch

```bash
git push origin HEAD:refs/heads/main
```

Never `--force`. On rejection: stop (full re-run after sync).

### 7. Publish (script owns tag + Release)

```bash
bash scripts/ship.sh publish \
  --expected-sha "$EXPECTED_SHA" \
  ${DRAFT:+--draft} \
  ${PORTABLE:+--portable}
```

This sets `SHIP_REQUIRE_CLEAN=1` and `SHIP_EXPECTED_SHA`, runs
`make publish-release` (or `-all`), and lets **publish-release** create/push
`v$VERSION` only after package+sign.

### 8. Report

```bash
gh release view "v${RELEASE_VERSION}" --json url,tagName,isDraft,assets \
  --jq '{url,tagName,isDraft,assets:[.assets[].name]}'
```

Report: version, tag, commit range, bump action, Release URL, asset names,
warnings.

## Never

- Create/push git tags from this skill
- `--skip-build` from ship (not exposed)
- Auto-stash / ship from non-`main`
- Hand-edit `Cargo.lock` version lines
- Force-push or `--no-verify`
- Continue after failed preflight or SHA drift

## Failure recovery

| Failed at | Recovery |
|-----------|----------|
| commit / preflight | Fix locally; re-run `ship` |
| push rejected | fetch + full ship (not push-only retry) |
| package/sign after push | `bash scripts/ship.sh publish --expected-sha <sha>` |
| tag exists elsewhere | `ship patch` or exact new version |
| draft release | `gh release edit vX.Y.Z --draft=false` when ready |

## Related

- `scripts/ship.sh` — plan / apply-version / preflight / publish / lock
- `scripts/publish-release.sh` — package, Sparkle, tag, `gh release`
- `git-commit` skill — every commit this pipeline creates
