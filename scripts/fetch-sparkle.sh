#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SPARKLE_ROOT="${SPARKLE_ROOT:-$ROOT/.sparkle}"
VERSION="2.9.4"
SHA256="ce89daf967db1e1893ed3ebd67575ed82d3902563e3191ca92aaec9164fbdef9"
ARCHIVE="Sparkle-${VERSION}.tar.xz"
URL="https://github.com/sparkle-project/Sparkle/releases/download/${VERSION}/${ARCHIVE}"

if [[ -f "$SPARKLE_ROOT/version" ]] \
  && [[ "$(<"$SPARKLE_ROOT/version")" == "$VERSION" ]] \
  && [[ -d "$SPARKLE_ROOT/Sparkle.framework" ]] \
  && [[ -x "$SPARKLE_ROOT/bin/sign_update" ]]; then
  exit 0
fi

work="$(mktemp -d "${TMPDIR:-/tmp}/kimini-sparkle.XXXXXX")"
trap 'rm -rf "$work"' EXIT

echo "==> download Sparkle ${VERSION}"
curl --fail --location --silent --show-error "$URL" --output "$work/$ARCHIVE"
echo "${SHA256}  $work/$ARCHIVE" | shasum -a 256 -c -
tar -xf "$work/$ARCHIVE" -C "$work"

rm -rf "$SPARKLE_ROOT"
mkdir -p "$SPARKLE_ROOT/bin"
ditto "$work/Sparkle.framework" "$SPARKLE_ROOT/Sparkle.framework"
for tool in generate_appcast generate_keys sign_update; do
  cp "$work/bin/$tool" "$SPARKLE_ROOT/bin/$tool"
  chmod 755 "$SPARKLE_ROOT/bin/$tool"
done
printf '%s' "$VERSION" > "$SPARKLE_ROOT/version"

echo "    cached at $SPARKLE_ROOT"
