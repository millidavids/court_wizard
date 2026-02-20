#!/bin/bash
set -e

# Check for --release or --profile flag
RELEASE_FLAG=""
BUILD_TYPE="debug"
OUT_DIR="./web"
PROFILE_NAME=""

SKIP_BUMP=false

for arg in "$@"; do
    case "$arg" in
        --release)
            RELEASE_FLAG="--release"
            BUILD_TYPE="release"
            PROFILE_NAME="release"
            OUT_DIR="./docs"
            SKIP_BUMP=true
            ;;
        --no-bump)
            SKIP_BUMP=true
            ;;
    esac
done

if [ "$PROFILE_NAME" = "release" ]; then
    echo "Building for WASM (RELEASE MODE - for GitHub Pages)..."
    echo "Skipping version bump (update CHANGELOG.md manually before release builds)"
elif [ "$SKIP_BUMP" = true ]; then
    echo "Building for WASM (debug mode - skipping version bump)..."
else
    # Bump patch version in Cargo.toml (debug builds only)
    echo "Bumping version..."
    CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

    # Parse semantic version
    MAJOR=$(echo $CURRENT_VERSION | cut -d. -f1)
    MINOR=$(echo $CURRENT_VERSION | cut -d. -f2)
    PATCH=$(echo $CURRENT_VERSION | cut -d. -f3)

    # Increment patch version
    NEW_PATCH=$((PATCH + 1))
    NEW_VERSION="$MAJOR.$MINOR.$NEW_PATCH"

    # Update Cargo.toml
    perl -pi -e "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml

    echo "Version bumped: $CURRENT_VERSION -> $NEW_VERSION"
    echo "Building for WASM (debug mode - for local testing)..."
fi

DEBUG_FEATURES=""
if [ "$BUILD_TYPE" = "debug" ]; then
    DEBUG_FEATURES="--features bevy/debug"
fi

cargo build --target wasm32-unknown-unknown $DEBUG_FEATURES $RELEASE_FLAG

echo "Running wasm-bindgen..."
wasm-bindgen \
  --out-dir $OUT_DIR \
  --out-name court_wizard \
  --target web \
  ./target/wasm32-unknown-unknown/$BUILD_TYPE/court_wizard.wasm

# Apply wasm-opt based on build type
if command -v wasm-opt &> /dev/null; then
    if [ "$PROFILE_NAME" = "release" ] || [ "$PROFILE_NAME" = "wasm-release" ]; then
        echo "Running wasm-opt for maximum size reduction..."
        # -Oz: Most aggressive size optimization for releases
        wasm-opt -Oz \
            --enable-nontrapping-float-to-int \
            --enable-bulk-memory \
            --enable-sign-ext \
            --enable-mutable-globals \
            --enable-reference-types \
            --enable-simd \
            --enable-multivalue \
            $OUT_DIR/court_wizard_bg.wasm \
            -o $OUT_DIR/court_wizard_bg.wasm
        echo "wasm-opt optimization complete!"
    else
        echo "Skipping wasm-opt for development builds (to avoid table growth issues)..."
        # Skip wasm-opt entirely for dev builds - the raw WASM from cargo is fine
    fi
else
    echo "Warning: wasm-opt not found, skipping post-processing optimization"
    echo "Install with: cargo install wasm-opt"
fi

if [ "$PROFILE_NAME" = "release" ] || [ "$PROFILE_NAME" = "wasm-release" ]; then
    echo "Copying index.html to docs/..."
    cp ./web/index.html ./docs/index.html 2>/dev/null || true
    echo "WASM build complete! Release files are in ./docs/ for GitHub Pages deployment."
    echo "Final binary size: $(ls -lh $OUT_DIR/court_wizard_bg.wasm | awk '{print $5}')"
else
    echo "WASM build complete! Debug files are in ./web/ for local testing."
    echo "Run ./serve.sh to test locally."
fi
