#!/bin/bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$DIR"

APP_NAME="DexHowTo HealthTask"
EXECUTABLE="HealthTasks" # cargo binary name
VERSION="$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"

if [ "${UNIVERSAL:-0}" = "1" ]; then
    # Apple Silicon + Intel in one binary (needs: rustup target add x86_64-apple-darwin)
    echo "Building ${APP_NAME} ${VERSION} (release, universal)..."
    cargo build --release --target aarch64-apple-darwin
    cargo build --release --target x86_64-apple-darwin
    mkdir -p target/release
    lipo -create \
        "target/aarch64-apple-darwin/release/${EXECUTABLE}" \
        "target/x86_64-apple-darwin/release/${EXECUTABLE}" \
        -output "target/release/${EXECUTABLE}"
else
    echo "Building ${APP_NAME} ${VERSION} (release)..."
    cargo build --release
fi
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

# Ad-hoc sign the whole bundle so the signature covers Info.plist and resources.
codesign --force --deep --sign - "${APP_BUNDLE}"

if [ "${UNIVERSAL:-0}" = "1" ]; then
    ZIP="target/release/DexHowTo-HealthTask-${VERSION}-macOS-universal.zip"
    rm -f "${ZIP}"
    ditto -c -k --sequesterRsrc --keepParent "${APP_BUNDLE}" "${ZIP}"
    echo "Release zip: ${ZIP}"
fi

echo "${APP_NAME}.app successfully created at: ${APP_BUNDLE}"
echo "You can launch it via: open \"${APP_BUNDLE}\""
