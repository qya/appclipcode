#!/usr/bin/env bash
set -euo pipefail

echo "==> Building WebAssembly module (release)..."
cargo build -p appclipcode-wasm --target wasm32-unknown-unknown --release

echo "==> Copying WASM to examples/html/ and examples/marko-vite/public/..."
mkdir -p examples/html examples/marko-vite/public
cp target/wasm32-unknown-unknown/release/appclipcode_wasm.wasm examples/html/
cp target/wasm32-unknown-unknown/release/appclipcode_wasm.wasm examples/marko-vite/public/

echo "==> WASM build complete: examples/html/appclipcode_wasm.wasm ($(ls -lh examples/html/appclipcode_wasm.wasm | awk '{print $5}'))"
