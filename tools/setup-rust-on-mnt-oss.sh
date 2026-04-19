#!/usr/bin/env bash
set -euo pipefail

# Minimal, VFS-anchored Rust toolchain setup for ecosafety-core

# 1. Point Rustup/Cargo to your VFS
OSS_ROOT="/mnt/oss"                       # change if your mount differs
export RUSTUP_HOME="${OSS_ROOT}/rustup"
export CARGO_HOME="${OSS_ROOT}/cargo"

# Ensure directories exist
mkdir -p "${RUSTUP_HOME}" "${CARGO_HOME}/bin"

# 2. Put Cargo on PATH for this shell
case ":$PATH:" in
  *":${CARGO_HOME}/bin:"*) ;;
  *) export PATH="${CARGO_HOME}/bin:${PATH}" ;;
esac

echo "[setup-rust-on-mnt-oss] RUSTUP_HOME=${RUSTUP_HOME}"
echo "[setup-rust-on-mnt-oss] CARGO_HOME=${CARGO_HOME}"
echo "[setup-rust-on-mnt-oss] PATH=${PATH}"

# 3. Install a minimal stable toolchain into mnt/oss
#    Use --profile=minimal to keep disk footprint low.
if ! command -v rustup >/dev/null 2>&1; then
  echo "[setup-rust-on-mnt-oss] Installing rustup (minimal profile)..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile=minimal --default-toolchain stable
else
  echo "[setup-rust-on-mnt-oss] rustup already present at $(command -v rustup)"
fi

# Re-source environment (rustup usually suggests this)
if [ -f "${CARGO_HOME}/env" ]; then
  # shellcheck disable=SC1090
  . "${CARGO_HOME}/env"
fi

# 4. Add rustfmt and clippy components to the stable toolchain
#    Components are added after installation, not via rustup-init args.[web:44][web:42]
echo "[setup-rust-on-mnt-oss] Adding rustfmt + clippy to stable toolchain..."
rustup toolchain install stable --profile=minimal
rustup default stable
rustup component add rustfmt
rustup component add clippy

# 5. Sanity checks
echo "[setup-rust-on-mnt-oss] Tool versions:"
rustc --version
cargo --version

# 6. Optional: first ecosafety-core build to confirm everything links
if [ -d "/workspace" ] && [ -f "/workspace/Cargo.toml" ]; then
  echo "[setup-rust-on-mnt-oss] Detected /workspace with Cargo.toml; testing ecosafety_core build..."
  cd /workspace
  if grep -q "ecosafety_core" Cargo.toml 2>/dev/null; then
    cargo build -p ecosafety_core
  else
    echo "[setup-rust-on-mnt-oss] ecosafety_core package not found in Cargo.toml; skipping targeted build."
    cargo build
  fi
else
  echo "[setup-rust-on-mnt-oss] No /workspace Cargo project detected; Rust install is ready for later use."
fi

echo "[setup-rust-on-mnt-oss] Done."
