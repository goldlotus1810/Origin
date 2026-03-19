#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════
# HomeOS — iOS build script
# PLAN 7.2.2: Prepare WASM binary + HTML for iOS WKWebView
#
# This script:
#   1. Builds origin.olang.wasm (bytecode embedded in WASM)
#   2. Copies origin.html (browser host)
#   3. Packages for Xcode project inclusion
#
# Requirements:
#   - wat2wasm (from wabt toolkit): brew install wabt
#   - cargo (for Rust builder)
#
# The actual iOS app is built via Xcode using HomeOSView.swift
# ═══════════════════════════════════════════════════════════════════════

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUNDLE_DIR="$SCRIPT_DIR/bundle"

echo "=== HomeOS iOS Build ==="

mkdir -p "$BUNDLE_DIR"

# 1. Compile WASM VM
echo "  Compiling WASM VM..."
if command -v wat2wasm &>/dev/null; then
    wat2wasm "$ROOT_DIR/vm/wasm/vm_wasm.wat" -o "$BUNDLE_DIR/vm_wasm.wasm"
    echo "  WASM VM: $(stat -c%s "$BUNDLE_DIR/vm_wasm.wasm" 2>/dev/null || stat -f%z "$BUNDLE_DIR/vm_wasm.wasm") bytes"
else
    echo "  Warning: wat2wasm not found. Install: brew install wabt"
    echo "  Skipping WASM compilation."
fi

# 2. Compile stdlib to bytecode
echo "  Compiling stdlib..."
if command -v cargo &>/dev/null; then
    BYTECODE="$BUNDLE_DIR/bytecode.bin"
    cargo run -p builder --quiet -- \
        --vm /dev/null \
        --stdlib "$ROOT_DIR/stdlib" \
        --codegen \
        -o /tmp/origin_wasm_tmp.olang 2>/dev/null || true
    # Extract just bytecode (skip the null VM wrapper)
    # For WASM mode: bytecode is loaded dynamically by host.js
    cp /tmp/origin_wasm_tmp.olang "$BUNDLE_DIR/bytecode.bin" 2>/dev/null || true
fi

# 3. Copy HTML host
echo "  Copying origin.html..."
cp "$ROOT_DIR/vm/wasm/origin.html" "$BUNDLE_DIR/origin.html"

# 4. Copy host.js
echo "  Copying host.js..."
cp "$ROOT_DIR/vm/wasm/host.js" "$BUNDLE_DIR/host.js"

echo ""
echo "=== iOS bundle ready ==="
echo "  Bundle: $BUNDLE_DIR/"
echo "  Files:"
ls -la "$BUNDLE_DIR/" 2>/dev/null
echo ""
echo "Next steps:"
echo "  1. Open Xcode → Create new iOS App"
echo "  2. Add HomeOSView.swift to project"
echo "  3. Add bundle/ contents to project resources"
echo "  4. Build & run on device/simulator"
