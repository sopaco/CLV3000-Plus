#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP_NAME="CLV3000 Plus"
BINARY="$ROOT/target/release/clv3000-plus"
APP="$ROOT/target/release/${APP_NAME}.app"
ICON_SRC="$ROOT/assets/icons/icon_app.icns"

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

cat >"$APP/Contents/Info.plist" <<'EOF'
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
  <string>0.1.0</string>
  <key>CFBundleVersion</key>
  <string>0.1.0</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
EOF

echo "Built $APP"
