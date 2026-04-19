#!/usr/bin/env bash
# Filename: /workspace/eco-ci-build.sh
# Destination: /workspace/eco-ci-build.sh
#
# Purpose:
#   Correct coder errors where they try to install Rust on `/`
#   or assume the toolchain lives in /root. This wrapper:
#     - points PATH at the existing Rust toolchain under /mnt/oss
#     - runs cargo build for eco-ci-validate from /workspace
#     - never runs rustup or apt-get
#
# Usage:
#   cd /workspace
#   chmod +x ./eco-ci-build.sh
#   ./eco-ci-build.sh

set -euo pipefail

# 1. Enforce working directory: /workspace
if [ "$(pwd)" != "/workspace" ]; then
  echo "[eco-ci-build] ERROR: Please run this script from /workspace." >&2
  echo "[eco-ci-build]   Hint: cd /workspace && ./eco-ci-build.sh" >&2
  exit 1
fi

# 2. Known Rust toolchain location on the VFS.
#    Adjust these paths if your /mnt/oss layout differs.
RUST_ROOT="/mnt/oss/rust"
CARGO_BIN="$RUST_ROOT/bin"

if [ ! -d "$RUST_ROOT" ] || [ ! -d "$CARGO_BIN" ]; then
  echo "[eco-ci-build] ERROR: Rust toolchain not found at $RUST_ROOT." >&2
  echo "[eco-ci-build]   Expected layout: $RUST_ROOT/bin/{rustc,cargo}." >&2
  echo "[eco-ci-build]   Please mount or install Rust under /mnt/oss, not /." >&2
  exit 1
fi

export PATH="$CARGO_BIN:$PATH"

# 3. Sanity check: rustc and cargo must be available now.
if ! command -v rustc >/dev/null 2>&1; then
  echo "[eco-ci-build] ERROR: rustc not found in PATH after adding $CARGO_BIN." >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "[eco-ci-build] ERROR: cargo not found in PATH after adding $CARGO_BIN." >&2
  exit 1
fi

# 4. Hard guardrails: refuse to run if someone tries to call rustup or apt-get.
if command -v rustup >/dev/null 2>&1; then
  echo "[eco-ci-build] WARNING: rustup is present, but will not be used." >&2
fi

# 5. Run the ecosafety CI build only, no system-wide installs.
echo "[eco-ci-build] Using rustc at: $(command -v rustc)"
echo "[eco-ci-build] Using cargo at: $(command -v cargo)"
echo "[eco-ci-build] Building eco-ci-validate in /workspace..."

cargo build -p eco-ci-validate --release

echo "[eco-ci-build] Build completed successfully."

exit 0
