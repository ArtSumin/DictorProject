#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FFI_DIR="$REPO_ROOT/DictorApp/Dependencies/DictorFFI"

BINDINGS=false
UNIVERSAL=false

for arg in "$@"; do
  case $arg in
    --bindings) BINDINGS=true ;;
    --universal) UNIVERSAL=true ;;
    -h|--help)
      echo "Usage: $0 [--bindings] [--universal]"
      echo "  --bindings   Regenerate Swift/FFI bindings (uniffi-bindgen)"
      echo "  --universal  Build universal binary (arm64 + x86_64) for macOS"
      exit 0
      ;;
  esac
done

cd "$REPO_ROOT/dictor-cli"
mkdir -p "$FFI_DIR"

if [ "$UNIVERSAL" = true ]; then
  echo "Building for aarch64-apple-darwin..."
  cargo build --release --target aarch64-apple-darwin
  echo "Building for x86_64-apple-darwin..."
  cargo build --release --target x86_64-apple-darwin
  echo "Creating universal binary..."
  lipo -create \
    -output "$FFI_DIR/libdictor.a" \
    target/aarch64-apple-darwin/release/libdictor.a \
    target/x86_64-apple-darwin/release/libdictor.a
  LIB_PATH="$FFI_DIR/libdictor.a"
  # uniffi-bindgen cannot read fat archives; use single-arch lib for bindings
  BINDINGS_LIB="$(pwd)/target/aarch64-apple-darwin/release/libdictor.a"
else
  cargo build --release
  # Find lib: release root first, then release/deps (hash-suffixed), then debug
  LIB_SRC=""
  if [ -f "target/release/libdictor.a" ]; then
    LIB_SRC="target/release/libdictor.a"
  else
    for f in target/release/deps/libdictor*.a; do
      if [ -f "$f" ]; then LIB_SRC="$f"; break; fi
    done
  fi
  if [ -z "$LIB_SRC" ]; then
    # Fallback: debug build (e.g. when CARGO_TARGET_DIR points elsewhere)
    for f in target/debug/deps/libdictor*.a; do
      if [ -f "$f" ]; then LIB_SRC="$f"; break; fi
    done
  fi
  if [ -z "$LIB_SRC" ] || [ ! -f "$LIB_SRC" ]; then
    echo "Error: libdictor.a not found in target/release/ or target/debug/deps/"
    exit 1
  fi
  cp "$LIB_SRC" "$FFI_DIR/libdictor.a"
  LIB_PATH="$FFI_DIR/libdictor.a"
fi

echo "✓ libdictor.a built and copied to Dependencies/DictorFFI/"

if [ "$BINDINGS" = true ]; then
  echo "Regenerating Swift bindings..."
  # uniffi-bindgen cannot read fat/universal archives; use single-arch lib when available
  BINDGEN_LIB="${BINDINGS_LIB:-$LIB_PATH}"
  cargo run --bin uniffi-bindgen -- generate --library "$BINDGEN_LIB" -l swift -o "$FFI_DIR"
  echo "✓ Bindings regenerated (dictor.swift, dictorFFI.h)"
fi
