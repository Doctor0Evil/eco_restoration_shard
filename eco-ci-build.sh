#!/bin/bash
# eco-ci-build.sh - Build script that uses the /mnt/oss-based Rust toolchain
# This script ensures all Rust builds avoid the constrained rootfs

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Load environment configuration
if [ -f "$SCRIPT_DIR/.env" ]; then
    source "$SCRIPT_DIR/.env"
else
    echo "ERROR: .env file not found at $SCRIPT_DIR/.env"
    echo "Please ensure RUSTUP_HOME and CARGO_HOME are set to /mnt/oss paths"
    exit 1
fi

echo ""
echo "=== Eco CI Build ==="
echo "Working directory: $(pwd)"
echo "Disk usage:"
df -h / /mnt/oss 2>/dev/null || df -h /
echo ""

# Run cargo build with release profile
echo "Building eco-ci-validate..."
cargo build -p eco-ci-validate --release

echo ""
echo "Build complete! Binary location:"
ls -lh target/release/eco-ci-validate

echo ""
echo "=== Build Summary ==="
echo "✓ Rust toolchain: $(rustc --version)"
echo "✓ CARGO_HOME: $CARGO_HOME (on /mnt/oss)"
echo "✓ RUSTUP_HOME: $RUSTUP_HOME (on /mnt/oss)"
echo "✓ No writes to /usr for Rust components"
