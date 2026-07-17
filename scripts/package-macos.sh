#!/usr/bin/env bash
# Build a signed-ad-hoc Kimini.app (and optional DMG / zip) for macOS.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: macOS packaging only runs on Darwin" >&2
  exit 1
fi

BIN_NAME="kimini"
APP_NAME="Kimini"
BUNDLE_ID="app.kimini"
DIST="${DIST:-$ROOT/dist}"
APP_DIR="$DIST/${APP_NAME}.app"
CONTENTS="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS/MacOS"
RES_DIR="$CONTENTS/Resources"
ICON_SRC="${ICON_SRC:-$ROOT/docs/brand/exports/app-icon-1024.png}"
PLIST_SRC="$ROOT/packaging/macos/Info.plist"
ICNS_CACHE="$ROOT/packaging/macos/AppIcon.icns"

# Optional cargo target triple, e.g. aarch64-apple-darwin / x86_64-apple-darwin
TARGET="${TARGET:-}"
# Artifact arch label: aarch64 | x86_64 (auto-detected from binary if empty)
ARCH_LABEL="${ARCH_LABEL:-}"

MAKE_DMG=0
MAKE_ZIP=0
SKIP_BUILD=0
INSTALL_DIR=""

usage() {
  cat <<EOF
Usage: $(basename "$0") [options]

  --dmg              Create a compressed DMG in dist/
  --zip              Create a .app zip archive in dist/
  --skip-build       Reuse an existing release binary
  --target TRIPLE    cargo --target (e.g. aarch64-apple-darwin)
  --arch LABEL       Artifact name arch (aarch64 | x86_64); default: detect
  --install DIR      Copy the .app to DIR after packaging
  --dist DIR         Output directory (default: ./dist)
  -h, --help         Show this help

Environment:
  VERSION            Override version string (default: Cargo.toml)
  TARGET / ARCH_LABEL  Same as --target / --arch
  DIST / ICON_SRC    Paths
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dmg) MAKE_DMG=1; shift ;;
    --zip) MAKE_ZIP=1; shift ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    --target)
      TARGET="${2:-}"
      if [[ -z "$TARGET" ]]; then
        echo "error: --target requires a triple" >&2
        exit 1
      fi
      shift 2
      ;;
    --arch)
      ARCH_LABEL="${2:-}"
      if [[ -z "$ARCH_LABEL" ]]; then
        echo "error: --arch requires a label" >&2
        exit 1
      fi
      shift 2
      ;;
    --install)
      INSTALL_DIR="${2:-}"
      if [[ -z "$INSTALL_DIR" ]]; then
        echo "error: --install requires a directory" >&2
        exit 1
      fi
      shift 2
      ;;
    --dist)
      DIST="${2:-}"
      if [[ -z "$DIST" ]]; then
        echo "error: --dist requires a directory" >&2
        exit 1
      fi
      APP_DIR="$DIST/${APP_NAME}.app"
      CONTENTS="$APP_DIR/Contents"
      MACOS_DIR="$CONTENTS/MacOS"
      RES_DIR="$CONTENTS/Resources"
      shift 2
      ;;
    -h|--help) usage; exit 0 ;;
    *)
      echo "error: unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ -n "${VERSION:-}" ]]; then
  :
else
  VERSION="$(
    awk -F'"' '/^version[[:space:]]*=/{print $2; exit}' Cargo.toml
  )"
fi
if [[ -z "$VERSION" ]]; then
  echo "error: could not read version (set VERSION= or fix Cargo.toml)" >&2
  exit 1
fi

# Resolve release binary path for host or cross target
if [[ -n "$TARGET" ]]; then
  REL_BIN="$ROOT/target/${TARGET}/release/$BIN_NAME"
else
  REL_BIN="$ROOT/target/release/$BIN_NAME"
fi

if [[ ! -f "$ICON_SRC" ]]; then
  echo "error: app icon not found: $ICON_SRC" >&2
  echo "  expected docs/brand/exports/app-icon-1024.png" >&2
  exit 1
fi

if [[ ! -f "$PLIST_SRC" ]]; then
  echo "error: Info.plist template missing: $PLIST_SRC" >&2
  exit 1
fi

detect_arch_label() {
  local bin="$1"
  local mach
  mach="$(file -b "$bin" 2>/dev/null || true)"
  case "$mach" in
    *arm64*|*ARM64*) echo "aarch64" ;;
    *x86_64*|*x86-64*) echo "x86_64" ;;
    *)
      # Fallback from TARGET triple
      case "${TARGET:-}" in
        aarch64-*) echo "aarch64" ;;
        x86_64-*) echo "x86_64" ;;
        *)
          case "$(uname -m)" in
            arm64) echo "aarch64" ;;
            x86_64) echo "x86_64" ;;
            *) echo "unknown" ;;
          esac
          ;;
      esac
      ;;
  esac
}

generate_icns() {
  local src="$1"
  local dest="$2"
  local work iconset
  work="$(mktemp -d "${TMPDIR:-/tmp}/kimini-icon.XXXXXX")"
  iconset="$work/AppIcon.iconset"
  mkdir -p "$iconset"

  local -a pairs=(
    "16:icon_16x16.png"
    "32:diana.s@example.org"
    "32:icon_32x32.png"
    "64:ivan.p@example.net"
    "128:icon_128x128.png"
    "256:wendy.h@example.net"
    "256:icon_256x256.png"
    "512:wendy.h@example.net"
    "512:icon_512x512.png"
    "1024:walt.e@example.net"
  )

  local pair size name
  for pair in "${pairs[@]}"; do
    size="${pair%%:*}"
    name="${pair#*:}"
    sips -z "$size" "$size" "$src" --out "$iconset/$name" >/dev/null
    sips -s format png "$iconset/$name" --out "$iconset/$name" >/dev/null 2>&1 || true
  done

  iconutil -c icns "$iconset" -o "$dest"
  rm -rf "$work"
}

# ---------------------------------------------------------------------------
# 1. Release binary
# ---------------------------------------------------------------------------
if [[ "$SKIP_BUILD" -eq 0 ]]; then
  # Prefer rustup-managed cargo so rust-std for --target is found.
  # Homebrew's rustc often lacks cross targets even when rustup has them.
  if [[ -x "${CARGO_HOME:-$HOME/.cargo}/bin/cargo" ]]; then
    export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
  fi
  if [[ -n "$TARGET" ]]; then
    echo "==> cargo build --release --target $TARGET"
    if command -v rustup >/dev/null 2>&1; then
      rustup target add "$TARGET"
    fi
    cargo build --release --target "$TARGET"
  else
    echo "==> cargo build --release"
    cargo build --release
  fi
elif [[ ! -x "$REL_BIN" ]]; then
  echo "error: --skip-build set but $REL_BIN is missing" >&2
  exit 1
fi

if [[ ! -x "$REL_BIN" ]]; then
  echo "error: release binary not found: $REL_BIN" >&2
  exit 1
fi

if [[ -z "$ARCH_LABEL" ]]; then
  ARCH_LABEL="$(detect_arch_label "$REL_BIN")"
fi
if [[ "$ARCH_LABEL" == "unknown" ]]; then
  echo "error: could not detect arch; pass --arch aarch64|x86_64" >&2
  exit 1
fi

ARTIFACT_STEM="${APP_NAME}-${VERSION}-macos-${ARCH_LABEL}"

# ---------------------------------------------------------------------------
# 2. AppIcon.icns (regenerate when source is newer)
# ---------------------------------------------------------------------------
need_icns=0
if [[ ! -f "$ICNS_CACHE" ]]; then
  need_icns=1
elif [[ "$ICON_SRC" -nt "$ICNS_CACHE" ]]; then
  need_icns=1
fi

if [[ "$need_icns" -eq 1 ]]; then
  echo "==> generate AppIcon.icns from $(basename "$ICON_SRC")"
  generate_icns "$ICON_SRC" "$ICNS_CACHE"
fi

# ---------------------------------------------------------------------------
# 3. Assemble .app
# ---------------------------------------------------------------------------
echo "==> assemble ${APP_NAME}.app (v${VERSION}, ${ARCH_LABEL})"
rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR" "$RES_DIR" "$DIST"

cp "$REL_BIN" "$MACOS_DIR/$BIN_NAME"
chmod 755 "$MACOS_DIR/$BIN_NAME"
cp "$ICNS_CACHE" "$RES_DIR/AppIcon.icns"

sed "s/@VERSION@/${VERSION}/g" "$PLIST_SRC" > "$CONTENTS/Info.plist"
printf 'APPL????' > "$CONTENTS/PkgInfo"

# ---------------------------------------------------------------------------
# 4. Ad-hoc codesign
# ---------------------------------------------------------------------------
echo "==> codesign (ad-hoc)"
codesign --force --deep --sign - --identifier "$BUNDLE_ID" "$APP_DIR"

if [[ ! -x "$MACOS_DIR/$BIN_NAME" ]] || [[ ! -f "$CONTENTS/Info.plist" ]]; then
  echo "error: incomplete bundle at $APP_DIR" >&2
  exit 1
fi

echo "    $APP_DIR"
echo "    arch: $ARCH_LABEL  binary: $(file -b "$MACOS_DIR/$BIN_NAME")"
du -sh "$APP_DIR" | awk '{print "    size: " $1}'
codesign -dv "$APP_DIR" 2>&1 | sed 's/^/    /' || true

# ---------------------------------------------------------------------------
# 5. Optional DMG
# ---------------------------------------------------------------------------
if [[ "$MAKE_DMG" -eq 1 ]]; then
  dmg_path="$DIST/${ARTIFACT_STEM}.dmg"
  stage="$(mktemp -d "${TMPDIR:-/tmp}/kimini-dmg.XXXXXX")"
  echo "==> create DMG $(basename "$dmg_path")"
  cp -R "$APP_DIR" "$stage/"
  ln -s /Applications "$stage/Applications"
  rm -f "$dmg_path"
  hdiutil create \
    -volname "${APP_NAME} ${VERSION} (${ARCH_LABEL})" \
    -srcfolder "$stage" \
    -ov -format UDZO \
    "$dmg_path" >/dev/null
  rm -rf "$stage"
  echo "    $dmg_path"
  du -sh "$dmg_path" | awk '{print "    size: " $1}'
fi

# ---------------------------------------------------------------------------
# 6. Optional zip (.app archive for GitHub Releases)
# ---------------------------------------------------------------------------
if [[ "$MAKE_ZIP" -eq 1 ]]; then
  zip_path="$DIST/${ARTIFACT_STEM}.zip"
  echo "==> create zip $(basename "$zip_path")"
  rm -f "$zip_path"
  # ditto preserves macOS metadata better than zip for .app bundles
  ditto -c -k --sequesterRsrc --keepParent "$APP_DIR" "$zip_path"
  echo "    $zip_path"
  du -sh "$zip_path" | awk '{print "    size: " $1}'
fi

# ---------------------------------------------------------------------------
# 7. Optional install
# ---------------------------------------------------------------------------
if [[ -n "$INSTALL_DIR" ]]; then
  mkdir -p "$INSTALL_DIR"
  dest="$INSTALL_DIR/${APP_NAME}.app"
  echo "==> install → $dest"
  rm -rf "$dest"
  cp -R "$APP_DIR" "$dest"
  codesign --force --deep --sign - --identifier "$BUNDLE_ID" "$dest"
  echo "    installed"
  echo
  echo "Launch:  open -a ${APP_NAME}"
  echo "First auth (if needed):"
  echo "  open -na ${APP_NAME} --args 'http://127.0.0.1:58627/#token=<daemon-token>'"
fi

echo
echo "Done. Bundle: $APP_DIR"
echo "Run:  open '$APP_DIR'"
echo "  or: open -na ${APP_NAME} --args 'http://127.0.0.1:58627/#token=…'"
