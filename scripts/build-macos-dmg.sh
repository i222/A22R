#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT_DIR/target/release/A22R.app"
DMG_PATH="$ROOT_DIR/target/release/A22R.dmg"
VOLUME_NAME="A22R"

if [[ "$(uname -s)" != "Darwin" ]]; then
    printf 'This packaging script requires macOS.\n' >&2
    exit 1
fi

"$ROOT_DIR/scripts/build-macos-app.sh"

if [[ ! -d "$APP_DIR" ]]; then
    printf 'Application bundle was not created: %s\n' "$APP_DIR" >&2
    exit 1
fi

STAGING="$(mktemp -d "${TMPDIR:-/tmp}/a22r-dmg.XXXXXX")"
cleanup() {
    rm -rf "$STAGING"
}
trap cleanup EXIT

ditto "$APP_DIR" "$STAGING/A22R.app"
ln -s /Applications "$STAGING/Applications"

if [[ -d "/Volumes/$VOLUME_NAME" ]]; then
    hdiutil detach "/Volumes/$VOLUME_NAME" -quiet || true
fi

rm -f "$DMG_PATH"
hdiutil create \
    -volname "$VOLUME_NAME" \
    -srcfolder "$STAGING" \
    -ov \
    -format UDZO \
    "$DMG_PATH" >/dev/null

printf 'Built macOS disk image:\n%s\n' "$DMG_PATH"
