#!/usr/bin/env bash
# Build reproducible portable Linux archives for Kimini and Kimini Web.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

APP="all"
TARGET=""
ARCH=""
DIST="${DIST:-$ROOT/dist}"
SKIP_BUILD=0

usage() {
  printf '%s\n' \
    "Usage: $(basename "$0") [options]" \
    "" \
    "  --app native|web|all  Application to package (default: all)" \
    "  --target TRIPLE       Cargo Linux target (default: rustc host)" \
    "  --arch ARCH           Artifact label: x86_64 or aarch64" \
    "  --skip-build          Reuse existing release binaries" \
    "  -h, --help            Show this help"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --app) APP="${2:-}"; shift 2 ;;
    --target) TARGET="${2:-}"; shift 2 ;;
    --arch) ARCH="${2:-}"; shift 2 ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) printf 'error: unknown option: %s\n' "$1" >&2; usage >&2; exit 1 ;;
  esac
done

if [[ "$(uname -s)" != "Linux" ]]; then
  printf 'error: package-linux.sh must run on Linux; use package-linux-docker.sh elsewhere\n' >&2
  exit 1
fi
if [[ "$APP" != "native" && "$APP" != "web" && "$APP" != "all" ]]; then
  printf 'error: --app must be native, web, or all\n' >&2
  exit 1
fi

HOST="$(rustc -vV | awk '/^host:/{print $2}')"
TARGET="${TARGET:-$HOST}"
if [[ "$TARGET" != *-unknown-linux-gnu ]]; then
  printf 'error: unsupported Linux target: %s\n' "$TARGET" >&2
  exit 1
fi
if [[ -z "$ARCH" ]]; then
  case "$TARGET" in
    x86_64-*) ARCH="x86_64" ;;
    aarch64-*) ARCH="aarch64" ;;
    *) printf 'error: cannot derive architecture from %s\n' "$TARGET" >&2; exit 1 ;;
  esac
fi
if [[ "$ARCH" != "x86_64" && "$ARCH" != "aarch64" ]]; then
  printf 'error: --arch must be x86_64 or aarch64\n' >&2
  exit 1
fi

VERSION="$(awk -F'"' '/^version[[:space:]]*=/{print $2; exit}' Cargo.toml)"
SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct 2>/dev/null || date +%s)}"
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
BIN_DIR="$TARGET_DIR/$TARGET/release"
mkdir -p "$DIST"

package_app() {
  local app="$1"
  local binary product desktop_id archive_name
  if [[ "$app" == "native" ]]; then
    binary="kimini"
    product="Kimini"
    desktop_id="io.github.reedchan7.Kimini"
  else
    binary="kimini-web"
    product="Kimini Web"
    desktop_id="io.github.reedchan7.KiminiWeb"
  fi
  archive_name="${product// /-}-${VERSION}-linux-${ARCH}"

  if [[ "$SKIP_BUILD" -eq 0 ]]; then
    if [[ "$app" == "native" ]]; then
      cargo build --locked --release --target "$TARGET" --bin "$binary" \
        --no-default-features --features native
    else
      cargo build --locked --release --target "$TARGET" --bin "$binary" \
        --no-default-features --features legacy-web
    fi
  fi
  if [[ ! -x "$BIN_DIR/$binary" ]]; then
    printf 'error: release binary not found: %s\n' "$BIN_DIR/$binary" >&2
    exit 1
  fi

  local staging
  staging="$(mktemp -d)"
  local bundle="$staging/$archive_name"
  mkdir -p \
    "$bundle/bin" \
    "$bundle/share/applications" \
    "$bundle/share/icons/hicolor/256x256/apps"
  install -m 0755 "$BIN_DIR/$binary" "$bundle/bin/$binary"
  install -m 0644 "packaging/linux/${desktop_id}.desktop" \
    "$bundle/share/applications/${desktop_id}.desktop"
  install -m 0644 docs/brand/exports/app-icon-256.png \
    "$bundle/share/icons/hicolor/256x256/apps/${desktop_id}.png"
  install -m 0644 LICENSE README.md "$bundle/"

  local archive="$DIST/${archive_name}.tar.gz"
  tar --sort=name --owner=0 --group=0 --numeric-owner \
    --mtime="@$SOURCE_DATE_EPOCH" -czf "$archive" -C "$staging" "$archive_name"
  local digest
  digest="$(sha256sum "$archive" | awk '{print $1}')"
  printf '%s  %s\n' "$digest" "$(basename "$archive")" > "$archive.sha256"
  rm -rf "$staging"
  printf 'created %s\n' "$archive"
}

if [[ "$APP" == "native" || "$APP" == "all" ]]; then
  package_app native
fi
if [[ "$APP" == "web" || "$APP" == "all" ]]; then
  package_app web
fi
