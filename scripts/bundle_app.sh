#!/bin/bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$DIR"

echo "Building DexHowTo HealthTask (release)..."
cargo build --release

APP_NAME="DexHowTo HealthTask"
EXECUTABLE="HealthTasks" # cargo binary name
APP_BUNDLE="target/release/${APP_NAME}.app"
CONTENTS_DIR="${APP_BUNDLE}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

echo "Creating bundle structure at ${APP_BUNDLE}..."
rm -rf "${APP_BUNDLE}"
mkdir -p "${MACOS_DIR}"
mkdir -p "${RESOURCES_DIR}"

# Copy binary
cp "target/release/${EXECUTABLE}" "${MACOS_DIR}/${EXECUTABLE}"
chmod +x "${MACOS_DIR}/${EXECUTABLE}"

# Copy app icon (regenerate with: swift scripts/make_icon.swift assets/icon_1024.png)
cp "assets/AppIcon.icns" "${RESOURCES_DIR}/AppIcon.icns"

# Info.plist (also embedded into the binary by build.rs)
cp "Info.plist" "${CONTENTS_DIR}/Info.plist"

echo "${APP_NAME}.app successfully created at: ${APP_BUNDLE}"
echo "You can launch it via: open \"${APP_BUNDLE}\""
