#!/bin/bash
set -e

# Check for --release flag
RELEASE_FLAG=""
BUILD_TYPE="debug"
if [ "$1" = "--release" ]; then
    RELEASE_FLAG="--release"
    BUILD_TYPE="release"
    echo "Building for WASM (RELEASE MODE)..."
else
    echo "Building for WASM (debug mode)..."
fi

cargo build --target wasm32-unknown-unknown $RELEASE_FLAG

echo "Running wasm-bindgen..."
wasm-bindgen \
  --out-dir ./web \
  --out-name the_game \
  --target web \
  ./target/wasm32-unknown-unknown/$BUILD_TYPE/the_game.wasm

echo "WASM build complete! Files are in ./web/"
echo "Open web/index.html in your browser to run the game."
