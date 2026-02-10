#!/usr/bin/env bash
set -e

# go to crate root (parent of utils/)
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

# get binary name from Cargo.toml
BIN_NAME=$(cargo metadata --no-deps --format-version 1 \
  | jq -r '.packages[0].targets[] | select(.kind[]=="bin") | .name')

OUT_DIR="$ROOT_DIR/dist"
mkdir -p "$OUT_DIR"

# macOS Apple Silicon
cargo build --release --target aarch64-apple-darwin
cp "target/aarch64-apple-darwin/release/$BIN_NAME" \
   "$OUT_DIR/${BIN_NAME}-macos-arm64"

# Windows
cargo build --release --target x86_64-pc-windows-gnu
cp "target/x86_64-pc-windows-gnu/release/${BIN_NAME}.exe" \
   "$OUT_DIR/${BIN_NAME}-windows-x86_64.exe"

# Arch Linux
cargo build --release --target x86_64-unknown-linux-gnu
cp "target/x86_64-unknown-linux-gnu/release/$BIN_NAME" \
   "$OUT_DIR/${BIN_NAME}-linux-x86_64"

echo "Binaries generated in $OUT_DIR/"
