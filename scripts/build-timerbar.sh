#!/bin/bash
set -euo pipefail

# Build TimerBar.app from Swift Package Manager project
# Usage: ./scripts/build-timerbar.sh [--release]

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TIMERBAR_DIR="$PROJECT_ROOT/TimerBar"

# Parse args
BUILD_CONFIG="debug"
if [[ "${1:-}" == "--release" ]]; then
    BUILD_CONFIG="release"
fi

echo "Building TimerBar ($BUILD_CONFIG)..."

# Build with Swift Package Manager
cd "$TIMERBAR_DIR"
if [[ "$BUILD_CONFIG" == "release" ]]; then
    swift build -c release
    BUILD_DIR=".build/release"
else
    swift build
    BUILD_DIR=".build/debug"
fi

# Create app bundle structure
APP_NAME="TimerBar.app"
APP_DIR="$TIMERBAR_DIR/$APP_NAME"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR"

# Copy executable
cp "$TIMERBAR_DIR/$BUILD_DIR/TimerBar" "$MACOS_DIR/"

# Copy Info.plist
cp "$TIMERBAR_DIR/Resources/Info.plist" "$CONTENTS_DIR/"

echo "Built: $APP_DIR"

# For release, also create a zip
if [[ "$BUILD_CONFIG" == "release" ]]; then
    cd "$TIMERBAR_DIR"
    ZIP_NAME="TimerBar-$(uname -m).zip"
    rm -f "$ZIP_NAME"
    ditto -c -k --keepParent "$APP_NAME" "$ZIP_NAME"
    echo "Created: $TIMERBAR_DIR/$ZIP_NAME"
fi
