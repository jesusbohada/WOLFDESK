#!/usr/bin/env bash
# ==============================================================================
# Script para empaquetar WolfDesk en macOS (.app y .dmg)
# Compatible con: macOS Monterey, Ventura, Sonoma, Sequoia (Intel y Apple Silicon)
# ==============================================================================

set -e

APP_NAME="WolfDesk"
VERSION="1.2.0"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_PATH="${1:-$ROOT_DIR/target/release/wolfdesk}"
APP_BUNDLE="$ROOT_DIR/dist_macos/$APP_NAME.app"
DMG_NAME="WolfDesk-macOS-Universal.dmg"

echo "=== Empaquetando WolfDesk para macOS (.app & .dmg) ==="

if [ ! -f "$BIN_PATH" ]; then
    echo "[!] No se encontró el binario: $BIN_PATH"
    echo "    Compilando binario de release..."
    cargo build --release --bin wolfdesk
    BIN_PATH="$ROOT_DIR/target/release/wolfdesk"
fi

# 1. Crear estructura de carpetas de la App
rm -rf "$ROOT_DIR/dist_macos"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

# 2. Copiar ejecutable
cp "$BIN_PATH" "$APP_BUNDLE/Contents/MacOS/wolfdesk"
chmod 755 "$APP_BUNDLE/Contents/MacOS/wolfdesk"

# 3. Copiar assets/icono si existe
if [ -f "$ROOT_DIR/crates/client-core/assets/wolfdesk.png" ]; then
    cp "$ROOT_DIR/crates/client-core/assets/wolfdesk.png" "$APP_BUNDLE/Contents/Resources/wolfdesk.png"
fi

# 4. Crear Info.plist con permisos para Screen Recording y Accesibilidad
cat <<EOF > "$APP_BUNDLE/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>wolfdesk</string>
    <key>CFBundleIconFile</key>
    <string>wolfdesk.png</string>
    <key>CFBundleIdentifier</key>
    <string>com.wolfdesk.remotedesktop</string>
    <key>CFBundleName</key>
    <string>WolfDesk</string>
    <key>CFBundleDisplayName</key>
    <string>WolfDesk Pro</string>
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
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>NSScreenCaptureUsageDescription</key>
    <string>WolfDesk necesita permiso para capturar la pantalla durante sesiones remotas.</string>
    <key>NSAccessibilityUsageDescription</key>
    <string>WolfDesk necesita permisos de accesibilidad para permitir el control de ratón y teclado.</string>
</dict>
</plist>
EOF

echo "✓ Bundle $APP_NAME.app generado exitosamente."

# 5. Generar archivo .dmg si hdiutil está disponible
if command -v hdiutil >/dev/null 2>&1; then
    echo "Generando imagen de disco .dmg..."
    DMG_TMP="$ROOT_DIR/dist_macos/dmg_tmp"
    mkdir -p "$DMG_TMP"
    cp -R "$APP_BUNDLE" "$DMG_TMP/"
    ln -s /Applications "$DMG_TMP/Applications"

    rm -f "$ROOT_DIR/$DMG_NAME"
    hdiutil create -volname "WolfDesk Installer" -srcfolder "$DMG_TMP" -ov -format UDZO "$ROOT_DIR/$DMG_NAME"
    rm -rf "$DMG_TMP"
    echo "===================================================="
    echo " [✓] Archivo DMG listo: $DMG_NAME"
    echo "===================================================="
else
    echo "[i] hdiutil solo está disponible en macOS. Se generó la carpeta $APP_NAME.app."
fi
