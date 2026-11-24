#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT_DIR/target/release/A22R.app"
BINARY="$ROOT_DIR/target/release/a22-app"
INFO_PLIST="$ROOT_DIR/packaging/macos/Info.plist"

if [[ "$(uname -s)" != "Darwin" ]]; then
    printf 'This packaging script requires macOS.\n' >&2
    exit 1
fi

cd "$ROOT_DIR"
cargo +1.99.1 build -p a22-app --release --locked

rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"
cp "$BINARY" "$APP_DIR/Contents/MacOS/a22-app"
cp "$INFO_PLIST" "$APP_DIR/Contents/Info.plist"
chmod 755 "$APP_DIR/Contents/MacOS/a22-app"

codesign --force --deep --sign - "$APP_DIR" >/dev/null

printf 'Built macOS application bundle:\n%s\n' "$APP_DIR"
printf 'Launch it with:\nopen "%s"\n' "$APP_DIR"
