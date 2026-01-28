#!/bin/bash
set -e

# Check for --release flag
RELEASE_FLAG=""
BUILD_TYPE="debug"
OUT_DIR="./web"
if [ "$1" = "--release" ]; then
    RELEASE_FLAG="--release"
    BUILD_TYPE="release"
    OUT_DIR="./docs"
    echo "Building for WASM (RELEASE MODE - for GitHub Pages)..."
else
    echo "Building for WASM (debug mode - for local testing)..."
fi

cargo build --target wasm32-unknown-unknown $RELEASE_FLAG

echo "Running wasm-bindgen..."
wasm-bindgen \
  --out-dir $OUT_DIR \
  --out-name the_game \
  --target web \
  ./target/wasm32-unknown-unknown/$BUILD_TYPE/the_game.wasm

# Run wasm-opt AFTER wasm-bindgen (not before) to avoid 'env' import issues
if [ "$1" = "--release" ]; then
    if command -v wasm-opt &> /dev/null; then
        echo "Running wasm-opt for size optimization..."
        wasm-opt -Oz $OUT_DIR/the_game_bg.wasm -o $OUT_DIR/the_game_bg.wasm
    else
        echo "Warning: wasm-opt not found, skipping optimization"
    fi
fi

if [ "$1" = "--release" ]; then
    echo "WASM build complete! Release files are in ./docs/ for GitHub Pages deployment."
else
    echo "WASM build complete! Debug files are in ./web/ for local testing."
    echo "Run ./serve.sh to test locally."
fi
