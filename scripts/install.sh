#!/usr/bin/env bash
#
# install.sh — Install blazers from the latest GitHub release, or build from
# source if no prebuilt binary is published.
#
# Usage:
#   ./scripts/install.sh
#   INSTALL_DIR=/usr/local/bin ./scripts/install.sh
set -euo pipefail

REPO="hieudoanm/blazers"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

echo "Installing blazers..."

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64 | amd64) ARCH="x86_64" ;;
    arm64 | aarch64) ARCH="aarch64" ;;
esac

case "$OS" in
    linux)  TRIPLE="${ARCH}-unknown-linux-gnu" ;;
    darwin) TRIPLE="${ARCH}-apple-darwin" ;;
    *)      TRIPLE="x86_64-pc-windows-msvc" ;;
esac

BINARY_URL="https://github.com/${REPO}/releases/latest/download/blazers-${TRIPLE}"
[[ "$OS" != "linux" && "$OS" != "darwin" ]] && BINARY_URL="${BINARY_URL}.exe"

if command -v curl >/dev/null 2>&1; then
    if curl -fsSL "$BINARY_URL" -o /tmp/blazers 2>/dev/null; then
        chmod +x /tmp/blazers
        mkdir -p "$INSTALL_DIR"
        install -m 0755 /tmp/blazers "$INSTALL_DIR/blazers"
        rm -f /tmp/blazers
        echo "blazers installed to ${INSTALL_DIR}/blazers"
        "$INSTALL_DIR/blazers" --help >/dev/null
        exit 0
    fi
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "Rust toolchain not found. Please install Rust from https://rustup.rs"
    exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
cd "$TMP_DIR"
git clone --depth 1 "https://github.com/${REPO}.git" blazers
cd blazers
cargo build --release -p blazers
mkdir -p "$INSTALL_DIR"
install -m 0755 target/release/blazers "$INSTALL_DIR/blazers"
echo "blazers installed to ${INSTALL_DIR}/blazers"
"$INSTALL_DIR/blazers" --help >/dev/null
