#!/usr/bin/env bash
# Build Linux x86_64 and ARM64 archives locally with Docker Buildx.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
DIST="${DIST:-$ROOT/dist}"
ARCH="all"

usage() {
  printf '%s\n' \
    "Usage: $(basename "$0") [--arch x86_64|aarch64|all]" \
    "" \
    "Build portable Linux archives through Docker Buildx."
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --arch) ARCH="${2:-}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) printf 'error: unknown option: %s\n' "$1" >&2; usage >&2; exit 1 ;;
  esac
done
if [[ "$ARCH" != "x86_64" && "$ARCH" != "aarch64" && "$ARCH" != "all" ]]; then
  printf 'error: --arch must be x86_64, aarch64, or all\n' >&2
  exit 1
fi
command -v docker >/dev/null 2>&1 || { printf 'error: Docker is required\n' >&2; exit 1; }
docker buildx version >/dev/null
mkdir -p "$DIST"

build_arch() {
  local arch="$1" platform output
  case "$arch" in
    x86_64) platform="linux/amd64" ;;
    aarch64) platform="linux/arm64" ;;
  esac
  output="$(mktemp -d)"
  docker buildx build \
    --file Dockerfile.linux \
    --platform "$platform" \
    --target artifacts \
    --output "type=local,dest=$output" \
    .
  find "$output" -maxdepth 1 -type f -exec cp {} "$DIST/" \;
  rm -rf "$output"
}

if [[ "$ARCH" == "x86_64" || "$ARCH" == "all" ]]; then
  build_arch x86_64
fi
if [[ "$ARCH" == "aarch64" || "$ARCH" == "all" ]]; then
  build_arch aarch64
fi
