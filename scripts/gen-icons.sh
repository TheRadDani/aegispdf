#!/usr/bin/env bash
# Generate all Tauri-required icon sizes from the master brand SVG.
# Master source: assets/aegispdf_logo.svg  (1024×1024 — do not edit here)
#
# Requires: Inkscape (preferred) OR rsvg-convert/ImageMagick (fallback).
# Run from the repository root:
#   chmod +x scripts/gen-icons.sh && ./scripts/gen-icons.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$SCRIPT_DIR/.."
SVG="$ROOT/assets/aegispdf_logo.svg"   # canonical brand logo
OUT="$ROOT/src-tauri/icons"            # Tauri icon output directory

die() { echo "ERROR: $*" >&2; exit 1; }

[[ -f "$SVG" ]] || die "Brand SVG not found: $SVG"

render_svg() {
  local svg="$1" out="$2" size="$3"
  if command -v inkscape &>/dev/null; then
    inkscape --export-type=png \
             --export-width="$size" --export-height="$size" \
             --export-filename="$out" "$svg" 2>/dev/null
  elif command -v rsvg-convert &>/dev/null; then
    rsvg-convert -w "$size" -h "$size" -o "$out" "$svg"
  elif command -v convert &>/dev/null; then
    convert -background none -resize "${size}x${size}" "$svg" "$out"
  else
    die "No SVG renderer found. Install inkscape, librsvg2-bin, or imagemagick."
  fi
}

echo "==> Source logo : $SVG"
echo "==> Output dir  : $OUT"
echo "==> Generating PNG icons..."
mkdir -p "$OUT"

render_svg "$SVG" "$OUT/32x32.png"      32
render_svg "$SVG" "$OUT/128x128.png"    128
render_svg "$SVG" "$OUT/256x256.png"    256

# Retina / @2x (256×256 treated as 128@2x by Tauri)
cp "$OUT/256x256.png" "$OUT/128x128@2x.png"

echo "==> Building icon.ico (Windows multi-size ICO)"
if command -v convert &>/dev/null; then
  # Generate intermediate sizes
  render_svg "$SVG" "$OUT/16x16.png"   16
  render_svg "$SVG" "$OUT/48x48.png"   48
  render_svg "$SVG" "$OUT/64x64.png"   64
  convert "$OUT/16x16.png" \
          "$OUT/32x32.png" \
          "$OUT/48x48.png" \
          "$OUT/64x64.png" \
          "$OUT/128x128.png" \
          "$OUT/256x256.png" \
          "$OUT/icon.ico"
  echo "   icon.ico created (6 sizes)"
elif command -v icotool &>/dev/null; then
  render_svg "$SVG" "$OUT/16x16.png"  16
  render_svg "$SVG" "$OUT/48x48.png"  48
  icotool -c -o "$OUT/icon.ico" \
    "$OUT/16x16.png" "$OUT/32x32.png" "$OUT/48x48.png" \
    "$OUT/128x128.png" "$OUT/256x256.png"
else
  echo "   WARNING: ImageMagick not found; icon.ico not generated."
  echo "            Install imagemagick or icoutils, then re-run."
fi

echo "==> Building icon.icns (macOS)"
if command -v png2icns &>/dev/null; then
  render_svg "$SVG" "$OUT/16x16.png"    16
  render_svg "$SVG" "$OUT/32x32.png"    32
  render_svg "$SVG" "$OUT/48x48.png"    48
  render_svg "$SVG" "$OUT/128x128.png"  128
  render_svg "$SVG" "$OUT/256x256.png"  256
  render_svg "$SVG" "$OUT/512x512.png"  512
  png2icns "$OUT/icon.icns" \
    "$OUT/16x16.png" "$OUT/32x32.png" "$OUT/48x48.png" \
    "$OUT/128x128.png" "$OUT/256x256.png" "$OUT/512x512.png"
  echo "   icon.icns created"
elif command -v iconutil &>/dev/null; then
  # macOS native path
  ICONSET="$OUT/icon.iconset"
  mkdir -p "$ICONSET"
  for SIZE in 16 32 48 128 256 512; do
    render_svg "$SVG" "$ICONSET/icon_${SIZE}x${SIZE}.png" "$SIZE"
    render_svg "$SVG" "$ICONSET/icon_${SIZE}x${SIZE}@2x.png" "$((SIZE * 2))"
  done
  iconutil -c icns "$ICONSET" -o "$OUT/icon.icns"
  rm -rf "$ICONSET"
  echo "   icon.icns created"
else
  echo "   WARNING: No icns tool found; icon.icns not generated."
  echo "            On Linux: apt install icnsutils"
  echo "            On macOS: iconutil is built-in."
fi

echo ""
echo "==> Icons written to: $OUT"
ls -1 "$OUT"/*.{png,ico,icns} 2>/dev/null || true
echo ""
echo "Done. Run 'npm run tauri build' to create your distributables."
