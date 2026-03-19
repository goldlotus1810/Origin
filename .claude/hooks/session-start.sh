#!/bin/bash
set -euo pipefail

# Only run in remote (web) environments
if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

cd "$CLAUDE_PROJECT_DIR"

# Ensure Rust toolchain is available
if ! command -v cargo &> /dev/null; then
  echo "cargo not found, installing Rust toolchain..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$CLAUDE_ENV_FILE"
  export PATH="$HOME/.cargo/bin:$PATH"
fi

# Install clippy component if missing
rustup component add clippy 2>/dev/null || true

# Build workspace to cache dependencies (takes advantage of container caching)
cargo build --workspace 2>&1 || true

# Also build test artifacts so first test run is fast
cargo test --workspace --no-run 2>&1 || true
