#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP_NAME="CLV3000 Plus"
BINARY="$ROOT/target/release/clv3000-plus"
APP="$ROOT/target/release/${APP_NAME}.app"
DMG="$ROOT/target/release/${APP_NAME}.dmg"
STAGING="$ROOT/target/release/dmg-staging"
ICON_SRC="$ROOT/assets/icons/icon_app.icns"
VERSION="$(sed -n 's/^version = "\(.*\)"$/\1/p' "$ROOT/Cargo.toml" | head -1)"
if [[ -z "$VERSION" ]]; then
  echo "cannot read workspace version from Cargo.toml" >&2
  exit 1
fi

cd "$ROOT"
cargo build -p clv-app --release

if [[ ! -f "$ICON_SRC" ]]; then
  echo "missing icon: $ICON_SRC" >&2
  exit 1
fi

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

cp "$BINARY" "$APP/Contents/MacOS/clv3000-plus"
cp "$ICON_SRC" "$APP/Contents/Resources/icon_app.icns"

cat >"$APP/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>zh_CN</string>
  <key>CFBundleExecutable</key>
  <string>clv3000-plus</string>
  <key>CFBundleIconFile</key>
  <string>icon_app</string>
  <key>CFBundleIdentifier</key>
  <string>com.clv3000.plus</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>CLV3000 Plus</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>${VERSION}</string>
  <key>CFBundleVersion</key>
  <string>${VERSION}</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>NSDesktopFolderUsageDescription</key>
  <string>CLV3000 Plus 读取桌面文件夹以发现可清理的构建缓存和 Agent 项目。 / CLV3000 Plus reads your Desktop to find build caches and agent projects that can be cleaned.</string>
  <key>NSDocumentsFolderUsageDescription</key>
  <string>CLV3000 Plus 读取文稿文件夹以发现可清理的构建缓存和 Agent 项目。 / CLV3000 Plus reads your Documents folder to find build caches and agent projects that can be cleaned.</string>
  <key>NSDownloadsFolderUsageDescription</key>
  <string>CLV3000 Plus 读取下载文件夹以发现可清理的构建缓存和 Agent 项目。 / CLV3000 Plus reads your Downloads folder to find build caches and agent projects that can be cleaned.</string>
</dict>
</plist>
EOF

echo "Built $APP"

# Package .app into a compressed DMG for distribution.
rm -rf "$STAGING" "$DMG"
mkdir -p "$STAGING"
cp -R "$APP" "$STAGING/"
ln -s /Applications "$STAGING/Applications"

hdiutil create \
  -volname "$APP_NAME" \
  -srcfolder "$STAGING" \
  -ov \
  -format UDZO \
  "$DMG"

rm -rf "$STAGING"
echo "Built $DMG"
