#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

BIN_NAME="pinel"
OUT_DIR="$ROOT_DIR/dist"
mkdir -p "$OUT_DIR"

OS="$(uname -s)"
ARCH="$(uname -m)"

cargo build --release --bin "$BIN_NAME"

case "$OS" in 
  Linux)
    SRC="target/release/$BIN_NAME"
    DEST="$OUT_DIR/${BIN_NAME}-linux-${ARCH}"
    ;;
  Darwin)
    SRC="target/release/$BIN_NAME"
    if [[ "$ARCH" == "arm64" ]]; then
      DEST="$OUT_DIR/${BIN_NAME}-macos-arm64"
    else
      DEST="$OUT_DIR/${BIN_NAME}-macos-x84_64"
    fi 
    ;;
  MINGW*|MSYS*|CYGWIN*)
    SRC="target/release/${BIN_NAME}.exe"
    DEST="$OUT_DIR/${BIN_NAME}-windows-x86_64.exe"
    ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac
