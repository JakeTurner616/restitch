#!/bin/bash
set -e

APP_NAME="restitch"
VERSION="v0.1.2"
DIST_DIR="dist"

# Always build for Linux
TARGETS=(
  "x86_64-unknown-linux-gnu"
  "aarch64-unknown-linux-gnu"
)

# If we're on macOS, include macOS targets too
if [[ "$(uname)" == "Darwin" ]]; then
  TARGETS+=(
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
  )
  echo "🍏 Detected macOS — adding Darwin targets"
else
  echo "🐧 Non-macOS system — skipping Darwin targets"
fi

echo "🔧 Building $APP_NAME $VERSION for multiple platforms..."

# Clean previous output
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# Build for each target using `cross`
for TARGET in "${TARGETS[@]}"; do
  echo "🚧 Compiling for $TARGET..."
  cross build --release --target "$TARGET"

  OUTDIR="$DIST_DIR/${APP_NAME}-${VERSION}-${TARGET}"
  mkdir -p "$OUTDIR"

  # Copy binary, readme, and default config
  cp "target/$TARGET/release/$APP_NAME" "$OUTDIR/"
  cp README.md config_targets.toml "$OUTDIR/"
  chmod +x "$OUTDIR/$APP_NAME"

  # Package it
  tar -czf "$OUTDIR.tar.gz" -C "$DIST_DIR" "$(basename "$OUTDIR")"
  echo "📦 Packaged: $OUTDIR.tar.gz"
done

echo -e "\n✅ All builds completed. Distributables are in $DIST_DIR/"