#!/bin/bash
set -e

echo "Building for WASM..."
cargo build --target wasm32-unknown-unknown

echo "Running wasm-bindgen..."
wasm-bindgen \
  --out-dir ./web \
  --out-name the_game \
  --target web \
  ./target/wasm32-unknown-unknown/debug/the_game.wasm

echo "WASM build complete! Files are in ./web/"
echo "Open web/index.html in your browser to run the game."
