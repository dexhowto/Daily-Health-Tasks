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

# Generate Info.plist
cat << 'EOF' > "${CONTENTS_DIR}/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>DexHowTo HealthTask</string>
    <key>CFBundleDisplayName</key>
    <string>DexHowTo HealthTask</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>com.healthtasks.macos</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleExecutable</key>
    <string>HealthTasks</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSRequiresAquaSystemAppearance</key>
    <false/>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.healthcare-fitness</string>
</dict>
</plist>
EOF

echo "${APP_NAME}.app successfully created at: ${APP_BUNDLE}"
echo "You can launch it via: open \"${APP_BUNDLE}\""
