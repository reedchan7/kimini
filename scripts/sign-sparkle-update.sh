#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SPARKLE_ROOT="${SPARKLE_ROOT:-$ROOT/.sparkle}"
SPARKLE_ACCOUNT="${SPARKLE_ACCOUNT:-kimini.reedchan7}"
ARCHIVE="${1:-}"

if [[ -z "$ARCHIVE" || ! -f "$ARCHIVE" ]]; then
  echo "usage: $(basename "$0") UPDATE.zip" >&2
  exit 1
fi

bash "$ROOT/scripts/fetch-sparkle.sh"

temp_key_dir=""
key_file="${SPARKLE_PRIVATE_KEY_FILE:-}"

cleanup_temp_key() {
  if [[ -n "$temp_key_dir" && "$key_file" == "$temp_key_dir/private-key" ]]; then
    rm -P "$key_file" 2>/dev/null || true
    rmdir "$temp_key_dir" 2>/dev/null || true
  fi
}
trap cleanup_temp_key EXIT
trap 'exit 130' HUP INT TERM

if [[ -z "$key_file" ]]; then
  temp_key_dir="$(mktemp -d "${TMPDIR:-/tmp}/kimini-sparkle-key.XXXXXX")"
  chmod 700 "$temp_key_dir"
  key_file="$temp_key_dir/private-key"
  "$SPARKLE_ROOT/bin/generate_keys" --account "$SPARKLE_ACCOUNT" -x "$key_file" >/dev/null
  chmod 600 "$key_file"
elif [[ ! -f "$key_file" ]]; then
  echo "error: Sparkle private key missing: $key_file" >&2
  exit 1
fi

"$SPARKLE_ROOT/bin/sign_update" --ed-key-file "$key_file" "$ARCHIVE"
