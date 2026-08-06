#!/bin/bash
set -e

# Build native binaries for Windows, Linux, macOS, or current host platform.
# Usage:
#   ./scripts/build_native.sh                       # Build for host platform (bumps patch version)
#   ./scripts/build_native.sh windows               # Build for Windows
#   ./scripts/build_native.sh linux                 # Build for Linux
#   ./scripts/build_native.sh macos                 # Build for macOS (Apple Silicon)
#   ./scripts/build_native.sh macos-intel           # Build for macOS (Intel)
#   ./scripts/build_native.sh --no-bump             # Build without bumping version
#   ./scripts/build_native.sh --benchmarking        # bench-release profile w/ diagnostics (host)
#   ./scripts/build_native.sh windows --benchmarking # bench-release for Windows
#   ./scripts/build_native.sh --release             # Release build for host (no bump)
#   ./scripts/build_native.sh windows --release     # Release build for Windows
#   ./scripts/build_native.sh linux --release       # Release build for Linux
#   ./scripts/build_native.sh macos --release       # Release build for macOS (Apple Silicon)
#   ./scripts/build_native.sh macos-intel --release # Release build for macOS (Intel)

cd "$(dirname "$0")/.."

TARGET=""
PROFILE="dev"
CARGO_PROFILE_FLAG=""
CARGO_FEATURE_FLAG=""
NO_BUMP=false
BENCHMARKING=false

for arg in "$@"; do
    case "$arg" in
        windows)        TARGET="x86_64-pc-windows-gnu" ;;
        linux)          TARGET="x86_64-unknown-linux-gnu" ;;
        macos)          TARGET="aarch64-apple-darwin" ;;
        macos-intel)    TARGET="x86_64-apple-darwin" ;;
        --release)      PROFILE="release"; CARGO_PROFILE_FLAG="--release"; NO_BUMP=true ;;
        --benchmarking) BENCHMARKING=true; NO_BUMP=true ;;
        --no-bump)      NO_BUMP=true ;;
    esac
done

# --benchmarking forces the bench-release profile and benchmarking feature.
# Overrides --release if both passed. Never bumps version.
if [ "$BENCHMARKING" = true ]; then
    PROFILE="bench-release"
    CARGO_PROFILE_FLAG="--profile bench-release"
    CARGO_FEATURE_FLAG="--features benchmarking"
fi

# Bump patch version unless --release or --no-bump
if [ "$NO_BUMP" = false ]; then
    CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    MAJOR=$(echo "$CURRENT_VERSION" | cut -d. -f1)
    MINOR=$(echo "$CURRENT_VERSION" | cut -d. -f2)
    PATCH=$(echo "$CURRENT_VERSION" | cut -d. -f3)
    NEW_PATCH=$((PATCH + 1))
    NEW_VERSION="$MAJOR.$MINOR.$NEW_PATCH"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
    else
        sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
    fi
    echo "Version bumped: $CURRENT_VERSION -> $NEW_VERSION"
fi

# Map cargo profile name to its target subdirectory.
case "$PROFILE" in
    dev)           PROFILE_DIR="debug" ;;
    release)       PROFILE_DIR="release" ;;
    bench-release) PROFILE_DIR="bench-release" ;;
    *)             PROFILE_DIR="$PROFILE" ;;
esac

if [ -n "$TARGET" ]; then
    TARGET_FLAG="--target $TARGET"
    BIN_DIR="./target/$TARGET/$PROFILE_DIR"
else
    TARGET_FLAG=""
    BIN_DIR="./target/$PROFILE_DIR"
fi

# Determine binary name
case "$TARGET" in
    *windows*) BIN_NAME="court_wizard.exe" ;;
    *)         BIN_NAME="court_wizard" ;;
esac

echo "Building native binary..."
echo "  Target: ${TARGET:-host}"
echo "  Profile: $PROFILE"
if [ "$BENCHMARKING" = true ]; then
    echo "  Features: benchmarking (diagnostics + F4 toggle ON)"
fi

cargo build $TARGET_FLAG $CARGO_PROFILE_FLAG $CARGO_FEATURE_FLAG

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

# Install-root IGA manifests for Steam auto-discovery (see the helper).
./scripts/copy_iga_manifests.sh "$BIN_DIR/controller_config"
echo "IGA manifests mirrored to install-root controller_config/."

# Copy SPRITE_CREDITS.csv alongside binary for attribution
CREDITS_SRC="./docs/SPRITE_CREDITS.csv"
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
# Resolve which Steam library to copy. For a no-arg host build TARGET is empty,
# so fall back to the host OS so macOS hosts get the .dylib (not the Linux .so).
STEAM_PLATFORM="$TARGET"
if [ -z "$STEAM_PLATFORM" ]; then
    case "$OSTYPE" in
        darwin*) STEAM_PLATFORM="apple" ;;
        msys*|cygwin*|win*) STEAM_PLATFORM="windows" ;;
        *) STEAM_PLATFORM="linux" ;;
    esac
fi
if [ -n "$STEAM_BUILD_DIR" ]; then
    case "$STEAM_PLATFORM" in
        *windows*)
            STEAM_DLL="$STEAM_BUILD_DIR/steam_api64.dll"
            if [ -f "$STEAM_DLL" ]; then
                cp "$STEAM_DLL" "$BIN_DIR/"
                echo "Steam API DLL copied."
            fi
            ;;
        *darwin*|*apple*)
            STEAM_DYLIB="$STEAM_BUILD_DIR/libsteam_api.dylib"
            if [ -f "$STEAM_DYLIB" ]; then
                cp "$STEAM_DYLIB" "$BIN_DIR/"
                echo "Steam API dynamic library copied."
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

# steam_appid.txt next to the binary lets a LOCAL build connect to the running
# Steam client (Steam Input, achievements, …) without going through Steam's
# library install — run the exe directly with Steam open, or drop the build into
# the app's install folder to launch it through Steam.
#
# DEBUG (`dev`) builds ONLY. NEVER for `release`: that binary ships to the Steam
# depots, where a bundled steam_appid.txt would FORCE one hardcoded app id on
# every player — breaking Steam features for anyone whose license is for a
# different id (a separate demo or playtest app). This has bitten us before.
# Release builds must use the app id Steam provides; `src/steam/init.rs` is the
# matching guard on the code side.
if [ "$PROFILE" = "dev" ]; then
    if [ -f "./steam_appid.txt" ]; then
        cp "./steam_appid.txt" "$BIN_DIR/"
    else
        # Fall back to the main app id (matches APP_ID in src/steam/constants.rs)
        # if the git-ignored root file is absent (e.g. a fresh checkout).
        echo "4550880" > "$BIN_DIR/steam_appid.txt"
    fi
    echo "steam_appid.txt copied (debug only — local Steam connect, no deploy)."
fi

echo ""
echo "To run:"
if [[ "$TARGET" == *"windows"* ]]; then
    echo "  Copy the whole $BIN_DIR/ folder (exe + assets/ + steam_api64.dll"
    echo "  + steam_appid.txt on debug) to Windows and run with Steam open."
    echo "  Or from WSL2: $BIN_DIR/$BIN_NAME"
else
    echo "  cd $BIN_DIR && ./$BIN_NAME"
fi
