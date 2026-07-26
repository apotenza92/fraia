#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$APP_DIR/build"
SOURCE="$BUILD_DIR/icon.svg"

for tool in sips iconutil; do
  command -v "$tool" >/dev/null 2>&1 || {
    echo "Required icon tool is unavailable: $tool" >&2
    exit 1
  }
done

test -f "$SOURCE" || {
  echo "Fraia icon source is missing: $SOURCE" >&2
  exit 1
}

TEMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/fraia-release-icons.XXXXXX")"
cleanup() {
  rm -rf -- "$TEMP_DIR"
}
trap cleanup EXIT

MASTER="$TEMP_DIR/icon-1024.png"
ICONSET="$TEMP_DIR/icon.iconset"
mkdir -p "$ICONSET" "$BUILD_DIR/icons"

sips --setProperty format png "$SOURCE" --out "$MASTER" >/dev/null

render_png() {
  local size="$1"
  local output="$2"
  sips --resampleHeightWidth "$size" "$size" "$MASTER" --out "$output" >/dev/null
}

render_png 16 "$ICONSET/icon_16x16.png"
render_png 32 "$ICONSET/icon_16x16@2x.png"
render_png 32 "$ICONSET/icon_32x32.png"
render_png 64 "$ICONSET/icon_32x32@2x.png"
render_png 128 "$ICONSET/icon_128x128.png"
render_png 256 "$ICONSET/icon_128x128@2x.png"
render_png 256 "$ICONSET/icon_256x256.png"
render_png 512 "$ICONSET/icon_256x256@2x.png"
render_png 512 "$ICONSET/icon_512x512.png"
cp "$MASTER" "$ICONSET/icon_512x512@2x.png"

iconutil --convert icns --output "$BUILD_DIR/icon.icns" "$ICONSET"
cp "$ICONSET/icon_512x512.png" "$BUILD_DIR/icons/512x512.png"
sips --setProperty format ico "$ICONSET/icon_256x256.png" --out "$BUILD_DIR/icon.ico" >/dev/null

for output in \
  "$BUILD_DIR/icon.icns" \
  "$BUILD_DIR/icon.ico" \
  "$BUILD_DIR/icons/512x512.png"; do
  test -s "$output" || {
    echo "Generated icon is missing or empty: $output" >&2
    exit 1
  }
done

echo "Generated maintained Fraia release icons from $SOURCE"
