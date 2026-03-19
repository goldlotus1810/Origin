#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════
# HomeOS — Android ARM64 build script
# PLAN 7.2.1: Build origin.olang for Android (ARM64 native)
#
# Two modes:
#   1. Termux (on-device): builds natively, no cross-compile needed
#   2. NDK (from Linux/Mac): cross-compiles for Android ARM64
#
# Requirements:
#   Termux: pkg install binutils  (provides as, ld)
#   NDK: $ANDROID_NDK_HOME set, or download from developer.android.com
# ═══════════════════════════════════════════════════════════════════════

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUTPUT="${1:-origin.olang}"

# Detect environment
if [ -d "/data/data/com.termux" ]; then
    MODE="termux"
    AS="as"
    LD="ld"
    echo "=== HomeOS Android Build (Termux native) ==="
elif [ -n "$ANDROID_NDK_HOME" ]; then
    MODE="ndk"
    TOOLCHAIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64"
    AS="$TOOLCHAIN/bin/aarch64-linux-android-as"
    LD="$TOOLCHAIN/bin/aarch64-linux-android-ld"
    echo "=== HomeOS Android Build (NDK cross-compile) ==="
else
    echo "Error: Need Termux or ANDROID_NDK_HOME"
    echo ""
    echo "Option 1 (Termux on device):"
    echo "  pkg install binutils"
    echo "  bash mobile/android/build.sh"
    echo ""
    echo "Option 2 (NDK cross-compile):"
    echo "  export ANDROID_NDK_HOME=/path/to/ndk"
    echo "  bash mobile/android/build.sh"
    exit 1
fi

# 1. Assemble ARM64 VM
echo "  Assembling VM (ARM64)..."
VM_SRC="$ROOT_DIR/vm/arm64/vm_arm64.S"
VM_OBJ="/tmp/vm_arm64.o"
VM_BIN="/tmp/vm_arm64"

if [ ! -f "$VM_SRC" ]; then
    echo "Error: $VM_SRC not found"
    exit 1
fi

$AS -o "$VM_OBJ" "$VM_SRC"
$LD -static -nostdlib -o "$VM_BIN" "$VM_OBJ"
echo "  VM: $(stat -c%s "$VM_BIN" 2>/dev/null || stat -f%z "$VM_BIN") bytes"

# 2. Compile stdlib
echo "  Compiling stdlib..."
if command -v cargo &>/dev/null; then
    # Use Rust builder if available
    cargo run -p builder --quiet -- \
        --vm "$VM_BIN" --wrap \
        --stdlib "$ROOT_DIR/stdlib" \
        --codegen \
        --arch arm64 \
        -o "$OUTPUT"
else
    echo "  Warning: cargo not available, using pre-built bytecode if exists"
    if [ -f "$ROOT_DIR/origin.olang" ]; then
        cp "$ROOT_DIR/origin.olang" "$OUTPUT"
    else
        echo "Error: Need cargo or pre-built origin.olang"
        exit 1
    fi
fi

chmod +x "$OUTPUT"
SIZE=$(stat -c%s "$OUTPUT" 2>/dev/null || stat -f%z "$OUTPUT")
echo ""
echo "=== Build complete ==="
echo "  Output: $OUTPUT ($SIZE bytes)"
echo "  Arch:   ARM64 (Android)"
echo ""
echo "To run:"
if [ "$MODE" = "termux" ]; then
    echo "  ./$OUTPUT"
else
    echo "  adb push $OUTPUT /data/local/tmp/"
    echo "  adb shell chmod +x /data/local/tmp/$OUTPUT"
    echo "  adb shell /data/local/tmp/$OUTPUT"
fi
