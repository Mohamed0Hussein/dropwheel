#!/usr/bin/env bash
# Cross-compiles the Windows app on Linux with MinGW, for when neither a Windows
# machine nor CI is at hand. The usual Windows build (MSVC) remains the reference.
#
# Needs, once:
#   sudo apt install mingw-w64 nsis
#   rustup target add x86_64-pc-windows-gnu
#   (cd ui && npm install)
#
# Run from anywhere:  scripts/build-windows-on-linux.sh
#
# Makes, under target/x86_64-pc-windows-gnu/release/:
#   bundle/nsis/Dropwheel_<version>_x64-setup.exe   the installer
#   Dropwheel_<version>_x64-portable.zip            dropwheel.exe and WebView2Loader.dll
#
# Unlike the MSVC build, a MinGW build loads WebView2Loader.dll at run time instead of
# linking it in, so the portable exe needs that DLL beside it. The installer adds it.

set -euo pipefail
cd "$(dirname "$0")/.."

TARGET=x86_64-pc-windows-gnu
OUT="target/$TARGET/release"

node scripts/fetch-pdfium.mjs --platform win-x64
(cd ui && npm run tauri build -- --target "$TARGET" --bundles nsis)

# Tauri copies the DLL next to the exe for MinGW builds.
loader="$OUT/WebView2Loader.dll"
if [[ ! -f "$loader" ]]; then
  echo "$loader is missing" >&2
  exit 1
fi
# The workspace version, which Tauri uses too.
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
zip="$OUT/Dropwheel_${version}_x64-portable.zip"
python3 - "$zip" "$OUT/dropwheel.exe" "$loader" <<'EOF'
import sys, zipfile
out, *files = sys.argv[1:]
with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
    for f in files:
        z.write(f, "Dropwheel/" + f.rsplit("/", 1)[-1])
EOF
echo "Portable: $zip"
ls "$OUT"/bundle/nsis/*.exe
