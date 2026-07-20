#!/usr/bin/env bash
# Deterministic helpers for the project `ship` skill.
# Agents orchestrate commits via git-commit; this script owns version math,
# preflight gates, and publish invocation (tagging stays in publish-release.sh).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Fixed product remote — not overridable (prevents mis-publish).
CANONICAL_REPO="reedchan7/kimini"
LOCK_FILE="${SHIP_LOCK_FILE:-$ROOT/.git/ship.lock}"
REMOTE="origin"
MAIN_REF="main"

usage() {
  cat <<'EOF'
Usage:
  scripts/ship.sh plan  [--bump default|patch|minor|major|X.Y.Z] [--portable] [--draft]
  scripts/ship.sh apply-version <X.Y.Z>
  scripts/ship.sh preflight
  scripts/ship.sh publish [--draft] [--portable] [--expected-sha <sha>]
  scripts/ship.sh lock
  scripts/ship.sh unlock
  scripts/ship.sh require-clean
  scripts/ship.sh require-main
  scripts/ship.sh require-canonical-remote
  scripts/ship.sh require-no-in-progress

plan is read-only (no commits, no version edits, no push/tag/publish).
EOF
}

die() {
  echo "error: $*" >&2
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"
}

# Read-only TOML field reads (never invoke cargo — plan must not rewrite lockfiles).
read_pkg_version() {
  awk -F'"' '/^version[[:space:]]*=/{print $2; exit}' Cargo.toml
}

read_pkg_name() {
  awk -F'"' '/^name[[:space:]]*=/{print $2; exit}' Cargo.toml
}

is_semver() {
  # Core X.Y.Z with optional -prerelease; no build metadata for releases.
  [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?$ ]]
}

semver_core() {
  local v="$1"
  printf '%s\n' "${v%%-*}"
}

bump_semver() {
  local mode="$1" version="$2"
  local core x y z
  core="$(semver_core "$version")"
  IFS=. read -r x y z <<<"$core"
  case "$mode" in
    patch) printf '%s.%s.%s\n' "$x" "$y" "$((z + 1))" ;;
    minor) printf '%s.%s.0\n' "$x" "$((y + 1))" ;;
    major) printf '%s.0.0\n' "$((x + 1))" ;;
    *) die "unknown bump mode: $mode" ;;
  esac
}

tag_commit() {
  local tag="$1"
  if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null 2>&1; then
    git rev-parse "refs/tags/${tag}^{commit}"
  else
    return 1
  fi
}

latest_tag_version() {
  # Highest local vX.Y.Z tag (numeric sort). Empty if none.
  git tag -l 'v[0-9]*' --sort=-v:refname \
    | sed -n 's/^v//p' \
    | while read -r v; do
        is_semver "$v" && { printf '%s\n' "$v"; break; }
      done
}

latest_github_release_version() {
  # Latest non-draft GitHub release tag (vX.Y.Z). Empty if gh unavailable/offline.
  command -v gh >/dev/null 2>&1 || return 0
  gh auth status >/dev/null 2>&1 || return 0
  GH_REPO="${CANONICAL_REPO}" gh release list --limit 30 --json tagName,isDraft \
    --jq '.[] | select(.isDraft|not) | .tagName' 2>/dev/null \
    | sed -n 's/^v//p' \
    | while read -r v; do
        is_semver "$v" && { printf '%s\n' "$v"; break; }
      done
}

latest_published_version() {
  local v
  v="$(latest_github_release_version || true)"
  if [[ -n "${v:-}" ]]; then
    printf '%s\n' "$v"
    return 0
  fi
  latest_tag_version
}

# Compare core X.Y.Z: echo -1 / 0 / 1 for a<b / a==b / a>b
semver_core_cmp() {
  local a b ax ay az bx by bz
  a="$(semver_core "$1")"
  b="$(semver_core "$2")"
  IFS=. read -r ax ay az <<<"$a"
  IFS=. read -r bx by bz <<<"$b"
  if (( ax != bx )); then
    (( ax > bx )) && { echo 1; return; }
    echo -1
    return
  fi
  if (( ay != by )); then
    (( ay > by )) && { echo 1; return; }
    echo -1
    return
  fi
  if (( az != bz )); then
    (( az > bz )) && { echo 1; return; }
    echo -1
    return
  fi
  echo 0
}

semver_gt() { [[ "$(semver_core_cmp "$1" "$2")" == "1" ]]; }
semver_ge() {
  local c
  c="$(semver_core_cmp "$1" "$2")"
  [[ "$c" == "1" || "$c" == "0" ]]
}

require_main() {
  local branch
  branch="$(git branch --show-current 2>/dev/null || true)"
  [[ "$branch" == "$MAIN_REF" ]] || die "ship only from branch '${MAIN_REF}' (current: ${branch:-detached})"
}

require_clean() {
  local st
  st="$(git status --porcelain=v1)" || die "git status failed"
  if [[ -n "$st" ]]; then
    printf '%s\n' "$st" >&2
    die "working tree must be clean before this step"
  fi
}

require_no_in_progress_git_op() {
  if [[ -d "$ROOT/.git/rebase-merge" || -d "$ROOT/.git/rebase-apply" \
     || -f "$ROOT/.git/MERGE_HEAD" || -f "$ROOT/.git/CHERRY_PICK_HEAD" \
     || -f "$ROOT/.git/REVERT_HEAD" ]]; then
    die "refuse ship while merge/rebase/cherry-pick/revert is in progress"
  fi
}

require_canonical_remote() {
  local url
  url="$(git remote get-url "$REMOTE" 2>/dev/null || true)"
  [[ -n "$url" ]] || die "missing git remote '${REMOTE}'"
  # Only github.com (ssh or https), exact owner/repo.
  if ! printf '%s' "$url" | rg -q '^(git@github\.com:|https://github\.com/)reedchan7/kimini(\.git)?/?$'; then
    die "remote '${REMOTE}' must be git@github.com:${CANONICAL_REPO}.git or https://github.com/${CANONICAL_REPO}.git (got: $url)"
  fi
}

require_gh_repo() {
  require_cmd gh
  gh auth status >/dev/null 2>&1 || die "gh is not authenticated (gh auth login)"
  if [[ -n "${GH_REPO:-}" && "${GH_REPO}" != "${CANONICAL_REPO}" ]]; then
    die "GH_REPO='${GH_REPO}' must be unset or equal to ${CANONICAL_REPO}"
  fi
}

acquire_lock() {
  mkdir -p "$(dirname "$LOCK_FILE")"
  if command -v python3 >/dev/null 2>&1; then
    python3 - "$LOCK_FILE" <<'PY'
import os, sys, time
path = sys.argv[1]
flags = os.O_CREAT | os.O_EXCL | os.O_WRONLY
try:
    fd = os.open(path, flags, 0o644)
except FileExistsError:
    # Stale lock > 2h: reclaim
    try:
        age = time.time() - os.path.getmtime(path)
    except OSError:
        age = 0
    if age > 7200:
        os.remove(path)
        fd = os.open(path, flags, 0o644)
    else:
        sys.stderr.write(f"error: ship lock held: {path}\n")
        sys.exit(1)
with os.fdopen(fd, "w") as f:
    f.write(f"pid={os.getpid()}\nstarted={time.time():.0f}\n")
PY
  else
    [[ ! -e "$LOCK_FILE" ]] || die "ship lock held: $LOCK_FILE"
    printf 'pid=%s\n' "$$" >"$LOCK_FILE"
  fi
}

release_lock() {
  rm -f "$LOCK_FILE"
}

cmd_lock() { acquire_lock; echo "locked $LOCK_FILE"; }
cmd_unlock() { release_lock; echo "unlocked $LOCK_FILE"; }

cmd_plan() {
  local bump_mode="default" portable=0 draft=0
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --bump) bump_mode="${2:-}"; shift 2 ;;
      --portable) portable=1; shift ;;
      --draft) draft=1; shift ;;
      -h|--help) usage; exit 0 ;;
      *) die "plan: unknown arg: $1" ;;
    esac
  done

  require_cmd git
  require_main
  require_no_in_progress_git_op
  # plan is read-only: no macOS/Sparkle/cargo/fetch required
  local name current origin_ver head_sha ahead behind tag_sha="" need_ship=0 action="keep" target="" reason=""
  name="$(read_pkg_name)"
  [[ "$name" == "kimini" ]] || die "package name is '${name}', expected kimini"
  current="$(read_pkg_version)"
  is_semver "$current" || die "current version is not semver: $current"

  # Read-only: never fetch. Use already-known origin/main + local tags.
  # (Real ship fetches in the skill before re-running plan.)
  if ! git show-ref --verify --quiet "refs/remotes/${REMOTE}/${MAIN_REF}"; then
    die "missing refs/remotes/${REMOTE}/${MAIN_REF}; run: git fetch ${REMOTE} ${MAIN_REF} --tags"
  fi

  origin_ver="$(git show "${REMOTE}/${MAIN_REF}:Cargo.toml" 2>/dev/null \
    | awk -F'"' '/^version[[:space:]]*=/{print $2; exit}' || true)"
  [[ -n "$origin_ver" ]] || origin_ver="$current"

  head_sha="$(git rev-parse HEAD)"
  ahead="$(git rev-list --count "${REMOTE}/${MAIN_REF}..HEAD" 2>/dev/null || echo 0)"
  behind="$(git rev-list --count "HEAD..${REMOTE}/${MAIN_REF}" 2>/dev/null || echo 0)"
  local dirty=0 dirty_st
  dirty_st="$(git status --porcelain=v1)" || die "git status failed"
  [[ -z "$dirty_st" ]] || dirty=1

  if tag_sha="$(tag_commit "v${current}" 2>/dev/null)"; then
    :
  else
    tag_sha=""
  fi

  local published release_exists=0 tag_at_head=0
  published="$(latest_published_version || true)"
  [[ -n "$published" ]] || published="$origin_ver"

  if [[ -n "$tag_sha" && "$tag_sha" == "$head_sha" ]]; then
    tag_at_head=1
  fi
  if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    if GH_REPO="${CANONICAL_REPO}" gh release view "v${current}" >/dev/null 2>&1; then
      release_exists=1
    fi
  fi

  # Shippable when there is local work, missing/wrong tag, or incomplete release.
  if [[ "$dirty" -eq 1 || "$ahead" -gt 0 ]]; then
    need_ship=1
  elif [[ "$tag_at_head" -eq 1 && "$release_exists" -eq 1 ]]; then
    need_ship=0
    reason="already released at v${current} (tag == HEAD, GH release exists)"
  elif [[ "$tag_at_head" -eq 1 && "$release_exists" -eq 0 ]]; then
    need_ship=1
    reason="tag v${current} at HEAD but GitHub Release missing (resume publish, keep version)"
  elif [[ -z "$tag_sha" ]]; then
    need_ship=1
    reason="no tag v${current} yet"
  elif [[ "$tag_sha" != "$head_sha" ]]; then
    need_ship=1
    reason="v${current} points at older commit ${tag_sha}"
  fi

  case "$bump_mode" in
    default)
      if [[ "$need_ship" -eq 0 ]]; then
        action="nothing"
        target="$current"
        reason="${reason:-already released at v${current}}"
      elif [[ "$tag_at_head" -eq 1 ]]; then
        # Incomplete release or dirty after tag: never bump again.
        action="keep"
        target="$current"
        reason="${reason:-resume publish at current version}"
      elif semver_gt "$current" "$published"; then
        # Version already advanced (e.g. bumped+pushed, tag not created yet).
        action="keep"
        target="$current"
        reason="current ${current} > published ${published}; keep for tag/release"
      elif [[ -n "$tag_sha" && "$tag_sha" != "$head_sha" ]]; then
        action="patch"
        target="$(bump_semver patch "$published")"
        reason="v${current} not at HEAD; patch from published ${published} → ${target}"
      else
        action="patch"
        target="$(bump_semver patch "$published")"
        reason="default patch from published ${published} → ${target}"
      fi
      ;;
    patch|minor|major)
      target="$(bump_semver "$bump_mode" "$published")"
      if [[ "$current" == "$target" ]]; then
        action="keep"
        reason="idempotent: already at target ${target} from published base ${published}"
      elif semver_gt "$current" "$target"; then
        action="keep"
        target="$current"
        reason="current ${current} already beyond ${bump_mode} target ${target}; keep"
      else
        action="$bump_mode"
        reason="bump from published ${published} → ${target}"
      fi
      ;;
    *)
      target="$bump_mode"
      is_semver "$target" || die "invalid exact version: $target"
      if [[ "$current" == "$target" ]]; then
        action="keep"
        reason="exact version already set"
      else
        action="exact"
        reason="set exact version ${target}"
      fi
      ;;
  esac

  if [[ "$action" == "nothing" ]]; then
    need_ship=0
  else
    need_ship=1
  fi

  local pub_target="publish-release" pub_flags="" publish_line
  [[ "$portable" -eq 1 ]] && pub_target="publish-release-all"
  [[ "$draft" -eq 1 ]] && pub_flags=" PUBLISH_FLAGS=--draft"
  publish_line="make ${pub_target}${pub_flags}"

  cat <<EOF
SHIP_PLAN
package=${name}
current_version=${current}
origin_version=${origin_ver}
published_base=${published}
head_sha=${head_sha}
ahead=${ahead}
behind=${behind}
dirty=${dirty}
current_tag_sha=${tag_sha:-}
bump_mode=${bump_mode}
action=${action}
target_version=${target}
need_ship=${need_ship}
portable=${portable}
draft=${draft}
reason=${reason}
publish_cmd=${publish_line}
EOF
}

cmd_apply_version() {
  local target="${1:-}"
  [[ -n "$target" ]] || die "apply-version requires X.Y.Z"
  is_semver "$target" || die "invalid version: $target"
  require_cmd cargo
  local current
  current="$(read_pkg_version)"
  if [[ "$current" == "$target" ]]; then
    echo "version already ${target}"
    return 0
  fi
  # Edit only the package version field under [package] (first version = line).
  python3 - "$target" <<'PY'
import pathlib, re, sys
target = sys.argv[1]
path = pathlib.Path("Cargo.toml")
text = path.read_text()
new, n = re.subn(
    r'(?m)^version\s*=\s*"[^"]*"',
    f'version = "{target}"',
    text,
    count=1,
)
if n != 1:
    raise SystemExit("error: could not rewrite package version in Cargo.toml")
path.write_text(new)
PY
  # Refresh Cargo.lock root package version via cargo (no hand-edits).
  cargo check --quiet --all-features --all-targets
  local now
  now="$(read_pkg_version)"
  [[ "$now" == "$target" ]] || die "Cargo.toml version is ${now}, expected ${target}"
  if ! awk -v target="$target" '
    $0 == "name = \"kimini\"" { in_pkg=1; next }
    in_pkg && /^version = / {
      if ($0 == "version = \"" target "\"") found=1
      in_pkg=0
    }
    END { exit found ? 0 : 1 }
  ' Cargo.lock; then
    die "Cargo.lock kimini version did not update to ${target}"
  fi
  echo "applied version ${target}"
}

cmd_preflight() {
  require_main
  require_clean
  require_cmd cargo
  echo "==> make check"
  make check
  echo "==> make test"
  make test
  require_clean
  echo "preflight ok"
}

cmd_publish() {
  local draft=0 portable=0 expected_sha=""
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --draft) draft=1; shift ;;
      --portable) portable=1; shift ;;
      --expected-sha) expected_sha="${2:-}"; shift 2 ;;
      *) die "publish: unknown arg: $1" ;;
    esac
  done

  [[ -n "$expected_sha" ]] || die "publish requires --expected-sha <sha>"
  [[ "$(uname -s)" == "Darwin" ]] || die "publish requires macOS"
  require_main
  require_no_in_progress_git_op
  require_clean
  require_canonical_remote
  require_gh_repo
  export GH_REPO="${CANONICAL_REPO}"

  local head_sha
  head_sha="$(git rev-parse HEAD)"
  if [[ "$expected_sha" != "$head_sha" ]]; then
    die "EXPECTED_SHA mismatch: expected ${expected_sha}, HEAD is ${head_sha}"
  fi

  local version tag tag_sha=""
  version="$(read_pkg_version)"
  tag="v${version}"
  if tag_sha="$(tag_commit "$tag" 2>/dev/null)"; then
    [[ "$tag_sha" == "$head_sha" ]] || die "tag ${tag} points at ${tag_sha}, not HEAD ${head_sha}"
  fi

  # Refuse clobber of a full published release at a different commit.
  if gh release view "$tag" >/dev/null 2>&1; then
    local is_draft
    is_draft="$(gh release view "$tag" --json isDraft -q .isDraft 2>/dev/null || echo false)"
    if [[ "$is_draft" != "true" && -n "$tag_sha" && "$tag_sha" == "$head_sha" ]]; then
      echo "warning: release ${tag} already exists (non-draft) at this SHA; will upload/clobber assets only via publish-release" >&2
    fi
  fi

  local flags=()
  [[ "$draft" -eq 1 ]] && flags+=(--draft)
  # Tag ownership: publish-release creates tag AFTER packaging succeeds.
  # Export for publish-release hardening.
  export SHIP_EXPECTED_SHA="$head_sha"
  export SHIP_REQUIRE_CLEAN=1
  export SHIP_REMOTE="$REMOTE"

  if [[ "$portable" -eq 1 ]]; then
    echo "==> publish-release-all (Windows assets must already be staged in dist/)"
    make publish-release-all PUBLISH_FLAGS="${flags[*]-}"
  else
    echo "==> publish-release"
    make publish-release PUBLISH_FLAGS="${flags[*]-}"
  fi

  # Final SHA assertion
  [[ "$(git rev-parse HEAD)" == "$head_sha" ]] || die "HEAD moved during publish"
  echo "publish complete for ${tag}"
  gh release view "$tag" --json url,tagName,isDraft,assets \
    --jq '{url,tagName,isDraft,assets:[.assets[].name]}' || true
}

main() {
  local cmd="${1:-}"
  shift || true
  case "$cmd" in
    plan) cmd_plan "$@" ;;
    apply-version) cmd_apply_version "$@" ;;
    preflight) cmd_preflight "$@" ;;
    publish) cmd_publish "$@" ;;
    lock) cmd_lock "$@" ;;
    unlock) cmd_unlock "$@" ;;
    require-clean) require_clean ;;
    require-main) require_main ;;
    require-canonical-remote) require_canonical_remote ;;
    require-no-in-progress) require_no_in_progress_git_op ;;
    -h|--help|"") usage; [[ -n "$cmd" ]] || exit 1 ;;
    *) die "unknown command: $cmd" ;;
  esac
}

main "$@"
