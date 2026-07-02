#!/bin/bash
set -e

# Package a release zip for the specified platform.
# Usage:
#   ./scripts/package.sh                          # Auto-detect platform from host
#   ./scripts/package.sh windows                  # Cross-compile for Windows
#   ./scripts/package.sh linux                    # Build for Linux
#   ./scripts/package.sh macos                    # Build for macOS (Apple Silicon)
#   ./scripts/package.sh macos-intel              # Build for macOS (Intel)
#   ./scripts/package.sh <platform> --skip-build  # Package from existing release build
#   ./scripts/package.sh --skip-build             # Auto-detect + skip build

cd "$(dirname "$0")/.."

SKIP_BUILD=false
PLATFORM=""

for arg in "$@"; do
    case "$arg" in
        --skip-build) SKIP_BUILD=true ;;
        windows|linux|macos|macos-intel) PLATFORM="$arg" ;;
        *)
            echo "Error: Unknown argument '$arg'."
            echo "Valid platforms: windows, linux, macos, macos-intel"
            echo "Valid flags: --skip-build"
            exit 1
            ;;
    esac
done

# Auto-detect platform if not specified
if [ -z "$PLATFORM" ]; then
    case "$(uname -s)" in
        Linux*) PLATFORM="linux" ;;
        Darwin*)
            if [ "$(uname -m)" = "arm64" ]; then
                PLATFORM="macos"
            else
                PLATFORM="macos-intel"
            fi
            ;;
        *)
            echo "Error: Could not auto-detect platform from '$(uname -s)'."
            echo "Specify one of: windows, linux, macos, macos-intel"
            exit 1
            ;;
    esac
    echo "Auto-detected platform: $PLATFORM"
fi

# Per-platform configuration
case "$PLATFORM" in
    windows)
        TARGET="x86_64-pc-windows-gnu"
        BIN_NAME="court_wizard.exe"
        STEAM_LIB="steam_api64.dll"
        ZIP_SUFFIX="windows"
        ;;
    linux)
        TARGET="x86_64-unknown-linux-gnu"
        BIN_NAME="court_wizard"
        STEAM_LIB="libsteam_api.so"
        ZIP_SUFFIX="linux"
        ;;
    macos)
        TARGET="aarch64-apple-darwin"
        BIN_NAME="court_wizard"
        STEAM_LIB="libsteam_api.dylib"
        ZIP_SUFFIX="macos-apple-silicon"
        ;;
    macos-intel)
        TARGET="x86_64-apple-darwin"
        BIN_NAME="court_wizard"
        STEAM_LIB="libsteam_api.dylib"
        ZIP_SUFFIX="macos-intel"
        ;;
esac

VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
BIN_DIR="./target/$TARGET/release"
ZIP_NAME="court_wizard-v${VERSION}-${ZIP_SUFFIX}.zip"
ZIP_PATH="$PWD/$ZIP_NAME"

if [ "$SKIP_BUILD" = false ]; then
    echo "Building $PLATFORM release..."
    ./scripts/build_native.sh "$PLATFORM" --release
fi

if [ ! -f "$BIN_DIR/$BIN_NAME" ]; then
    echo "Error: $BIN_DIR/$BIN_NAME not found. Run without --skip-build first."
    exit 1
fi

# Create a temporary staging directory
STAGING=$(mktemp -d)
trap 'rm -rf "$STAGING"' EXIT

mkdir -p "$STAGING/court_wizard"
cp "$BIN_DIR/$BIN_NAME" "$STAGING/court_wizard/"
cp -r "$BIN_DIR/assets" "$STAGING/court_wizard/"
cp docs/PLAYER_README.txt "$STAGING/court_wizard/README.txt"

# Steam auto-discovers the In-Game Actions manifest at the depot/install ROOT:
# controller_config/game_actions_<appid>.vdf. The .vdf files also live under
# assets/ (the runtime SetInputActionManifestFilePath fallback), but Steam only
# looks at the root — so ship a copy there too. Both the main app and the
# Playtest read the same depot, so we include every game_actions_*.vdf and each
# app picks the one named for its own id.
if [ -d "./assets/controller_config" ]; then
    mkdir -p "$STAGING/court_wizard/controller_config"
    cp ./assets/controller_config/*.vdf "$STAGING/court_wizard/controller_config/" 2>/dev/null || true
    echo "Included install-root controller_config/ (Steam IGA auto-discovery)."
fi

# Steam redistributable library
if [ -f "$BIN_DIR/$STEAM_LIB" ]; then
    cp "$BIN_DIR/$STEAM_LIB" "$STAGING/court_wizard/"
    echo "Included $STEAM_LIB"
else
    echo "Warning: $STEAM_LIB not found in $BIN_DIR — Steam features will not work."
fi

# Create zip (zip → 7z → python3 fallback)
if command -v zip &> /dev/null; then
    (cd "$STAGING" && zip -r -q "$ZIP_PATH" court_wizard)
elif command -v 7z &> /dev/null; then
    (cd "$STAGING" && 7z a -tzip -r "$ZIP_PATH" court_wizard) > /dev/null
elif command -v python3 &> /dev/null; then
    python3 -c "
import zipfile, os, sys
with zipfile.ZipFile(sys.argv[1], 'w', zipfile.ZIP_DEFLATED) as zf:
    for root, dirs, files in os.walk(sys.argv[2]):
        for f in files:
            full = os.path.join(root, f)
            zf.write(full, os.path.relpath(full, sys.argv[3]))
" "$ZIP_PATH" "$STAGING/court_wizard" "$STAGING"
else
    echo "Error: No zip tool found. Install zip, 7z, or python3."
    exit 1
fi

SIZE=$(du -h "$ZIP_NAME" | cut -f1)
echo ""
echo "Packaged: $ZIP_NAME ($SIZE)"
