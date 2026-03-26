#!/bin/bash
# Build HomeOS WASM và copy vào www/
set -e

echo "○ Building WASM..."
cd "$(dirname "$0")/.."

# Install wasm-pack nếu chưa có
if ! command -v wasm-pack &>/dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build
wasm-pack build crates/wasm --target web --out-dir ../../www/pkg

echo "○ WASM built → www/pkg/"
echo "  Serve: cd www && python3 -m http.server 8080"
echo "  Open:  http://localhost:8080"
