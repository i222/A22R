#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT_DIR/target/release/A22R.app"
BINARY="$ROOT_DIR/target/release/a22-app"
INFO_PLIST="$ROOT_DIR/packaging/macos/Info.plist"
ICON_SOURCE="$ROOT_DIR/packaging/macos/app-icon.svg"

if [[ "$(uname -s)" != "Darwin" ]]; then
    printf 'This packaging script requires macOS.\n' >&2
    exit 1
fi

ICON_WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/a22r-iconset.XXXXXX")"
cleanup() {
    rm -rf "$ICON_WORK_DIR"
}
trap cleanup EXIT

cd "$ROOT_DIR"
cargo +1.99.1 build -p a22-app --release --locked

rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"
cp "$BINARY" "$APP_DIR/Contents/MacOS/a22-app"
cp "$INFO_PLIST" "$APP_DIR/Contents/Info.plist"
chmod 755 "$APP_DIR/Contents/MacOS/a22-app"

ICONSET="$ICON_WORK_DIR/AppIcon.iconset"
mkdir -p "$ICONSET"
make_icon() {
    local size="$1"
    local filename="$2"
    sips -z "$size" "$size" -s format png "$ICON_SOURCE" \
        --out "$ICONSET/$filename" >/dev/null
}

make_icon 16 "icon_16x16.png"
make_icon 32 "icon_16x16@2x.png"
make_icon 32 "icon_32x32.png"
make_icon 64 "icon_32x32@2x.png"
make_icon 128 "icon_128x128.png"
make_icon 256 "icon_128x128@2x.png"
make_icon 256 "icon_256x256.png"
make_icon 512 "icon_256x256@2x.png"
make_icon 512 "icon_512x512.png"
make_icon 1024 "icon_512x512@2x.png"
iconutil -c icns "$ICONSET" -o "$APP_DIR/Contents/Resources/AppIcon.icns"

codesign --force --deep --sign - "$APP_DIR" >/dev/null

printf 'Built macOS application bundle:\n%s\n' "$APP_DIR"
printf 'Launch it with:\nopen "%s"\n' "$APP_DIR"
