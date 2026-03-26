#!/bin/bash
set -e

# Build native binaries for Windows, Linux, macOS, or current host platform.
# Usage:
#   ./build_native.sh                       # Build for host platform (bumps patch version)
#   ./build_native.sh windows               # Build for Windows
#   ./build_native.sh linux                 # Build for Linux
#   ./build_native.sh macos                 # Build for macOS (Apple Silicon)
#   ./build_native.sh macos-intel           # Build for macOS (Intel)
#   ./build_native.sh --no-bump             # Build without bumping version
#   ./build_native.sh --release             # Release build for host (no bump)
#   ./build_native.sh windows --release     # Release build for Windows
#   ./build_native.sh linux --release       # Release build for Linux
#   ./build_native.sh macos --release       # Release build for macOS (Apple Silicon)
#   ./build_native.sh macos-intel --release # Release build for macOS (Intel)

TARGET=""
PROFILE="dev"
CARGO_PROFILE_FLAG=""
NO_BUMP=false

for arg in "$@"; do
    case "$arg" in
        windows)     TARGET="x86_64-pc-windows-gnu" ;;
        linux)       TARGET="x86_64-unknown-linux-gnu" ;;
        macos)       TARGET="aarch64-apple-darwin" ;;
        macos-intel) TARGET="x86_64-apple-darwin" ;;
        --release)   PROFILE="release"; CARGO_PROFILE_FLAG="--release"; NO_BUMP=true ;;
        --no-bump)   NO_BUMP=true ;;
    esac
done

# Bump patch version unless --release or --no-bump
if [ "$NO_BUMP" = false ]; then
    CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    MAJOR=$(echo "$CURRENT_VERSION" | cut -d. -f1)
    MINOR=$(echo "$CURRENT_VERSION" | cut -d. -f2)
    PATCH=$(echo "$CURRENT_VERSION" | cut -d. -f3)
    NEW_PATCH=$((PATCH + 1))
    NEW_VERSION="$MAJOR.$MINOR.$NEW_PATCH"
    sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
    echo "Version bumped: $CURRENT_VERSION -> $NEW_VERSION"
fi

# Determine output directory based on target and profile
if [ -n "$TARGET" ]; then
    TARGET_FLAG="--target $TARGET"
    if [ "$PROFILE" = "release" ]; then
        BIN_DIR="./target/$TARGET/release"
    else
        BIN_DIR="./target/$TARGET/debug"
    fi
else
    TARGET_FLAG=""
    if [ "$PROFILE" = "release" ]; then
        BIN_DIR="./target/release"
    else
        BIN_DIR="./target/debug"
    fi
fi

# Determine binary name
case "$TARGET" in
    *windows*) BIN_NAME="court_wizard.exe" ;;
    *)         BIN_NAME="court_wizard" ;;
esac

echo "Building native binary..."
echo "  Target: ${TARGET:-host}"
echo "  Profile: $PROFILE"

cargo build $TARGET_FLAG $CARGO_PROFILE_FLAG

echo "Build complete: $BIN_DIR/$BIN_NAME"

# Copy assets alongside binary so Bevy can find them
ASSET_SRC="./assets"
ASSET_DST="$BIN_DIR/assets"

if [ -d "$ASSET_SRC" ]; then
    echo "Syncing assets to $ASSET_DST..."
    mkdir -p "$ASSET_DST"
    # Use rsync if available for incremental copies, otherwise cp
    if command -v rsync &> /dev/null; then
        rsync -a --delete "$ASSET_SRC/" "$ASSET_DST/"
    else
        cp -r "$ASSET_SRC/"* "$ASSET_DST/"
    fi
    echo "Assets synced."
else
    echo "Warning: $ASSET_SRC not found. Assets will be missing at runtime."
fi

# Copy SPRITE_CREDITS.csv alongside binary for attribution
CREDITS_SRC="./credits/SPRITE_CREDITS.csv"
if [ -f "$CREDITS_SRC" ]; then
    cp "$CREDITS_SRC" "$BIN_DIR/"
    echo "Sprite credits CSV copied."
fi

# Copy Steam redistributable DLLs/SOs alongside binary
if [ -n "$TARGET" ]; then
    STEAM_SEARCH_DIR="./target/$TARGET"
else
    STEAM_SEARCH_DIR="./target"
fi
STEAM_BUILD_DIR=$(find "$STEAM_SEARCH_DIR" -path "*/build/steamworks-sys-*/out" -type d 2>/dev/null | head -1)
if [ -n "$STEAM_BUILD_DIR" ]; then
    case "$TARGET" in
        *windows*)
            STEAM_DLL="$STEAM_BUILD_DIR/steam_api64.dll"
            if [ -f "$STEAM_DLL" ]; then
                cp "$STEAM_DLL" "$BIN_DIR/"
                echo "Steam API DLL copied."
            fi
            ;;
        *)
            STEAM_SO="$STEAM_BUILD_DIR/libsteam_api.so"
            if [ -f "$STEAM_SO" ]; then
                cp "$STEAM_SO" "$BIN_DIR/"
                echo "Steam API shared library copied."
            fi
            ;;
    esac
fi

echo ""
echo "To run:"
if [[ "$TARGET" == *"windows"* ]]; then
    echo "  Copy $BIN_DIR/$BIN_NAME and $ASSET_DST/ to a Windows machine and run."
    echo "  Or from WSL2: $BIN_DIR/$BIN_NAME"
else
    echo "  cd $BIN_DIR && ./$BIN_NAME"
fi
