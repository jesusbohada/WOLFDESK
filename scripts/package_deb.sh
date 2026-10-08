#!/usr/bin/env bash
# ==============================================================================
# Script para empaquetar WolfDesk en un archivo .deb (Ubuntu / Debian / Mint)
# ==============================================================================

set -e

VERSION="1.2.0"
ARCH="amd64"
PKG_DIR="target/debian/wolfdesk_${VERSION}_${ARCH}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "=== Generando paquete Debian (.deb) para WolfDesk v${VERSION} ==="

# 1. Compilar binario si no existe
if [ ! -f "$ROOT_DIR/target/release/wolfdesk" ]; then
    echo "Compilando WolfDesk en modo Release..."
    cargo build --release --bin wolfdesk
fi

# 2. Limpiar estructura previa
rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/bin"
mkdir -p "$PKG_DIR/usr/share/applications"
mkdir -p "$PKG_DIR/usr/share/icons/hicolor/256x256/apps"

# 3. Crear archivo de control DEBIAN
cat <<EOF > "$PKG_DIR/DEBIAN/control"
Package: wolfdesk
Version: ${VERSION}
Section: net
Priority: optional
Architecture: ${ARCH}
Depends: libc6 (>= 2.31), libx11-6, libasound2, libxcursor1, libxrandr2, libxi6
Maintainer: WolfDesk Software <soporte@wolfdesk.com>
Homepage: https://github.com/wolfdesk/remote-desktop
Description: WolfDesk Pro - Escritorio Remoto Seguro
 WolfDesk es una aplicacion de escritorio remoto rapida, segura y de bajisima
 latencia para control remoto, transferencia de archivos y soporte tecnico.
EOF

# 4. Crear script postinst
cat <<EOF > "$PKG_DIR/DEBIAN/postinst"
#!/bin/sh
set -e
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
fi
exit 0
EOF
chmod 755 "$PKG_DIR/DEBIAN/postinst"

# 5. Crear script postrm
cat <<EOF > "$PKG_DIR/DEBIAN/postrm"
#!/bin/sh
set -e
if [ "\$1" = "remove" ] || [ "\$1" = "purge" ]; then
    if [ "\$1" = "purge" ]; then
        rm -rf /etc/wolfdesk || true
    fi
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database /usr/share/applications || true
    fi
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
    fi
fi
exit 0
EOF
chmod 755 "$PKG_DIR/DEBIAN/postrm"

# 6. Copiar binario y assets
cp "$ROOT_DIR/target/release/wolfdesk" "$PKG_DIR/usr/bin/wolfdesk"
chmod 755 "$PKG_DIR/usr/bin/wolfdesk"

if [ -f "$ROOT_DIR/crates/client-core/assets/wolfdesk.png" ]; then
    cp "$ROOT_DIR/crates/client-core/assets/wolfdesk.png" "$PKG_DIR/usr/share/icons/hicolor/256x256/apps/wolfdesk.png"
fi

if [ -f "$ROOT_DIR/crates/client-core/assets/wolfdesk.desktop" ]; then
    cp "$ROOT_DIR/crates/client-core/assets/wolfdesk.desktop" "$PKG_DIR/usr/share/applications/wolfdesk.desktop"
fi

# 7. Empaquetar con dpkg-deb
DEB_NAME="wolfdesk_${VERSION}_${ARCH}.deb"
dpkg-deb --build --root-owner-group "$PKG_DIR" "$ROOT_DIR/$DEB_NAME"

echo "===================================================="
echo " [✓] Paquete Debian creado exitosamente: $DEB_NAME"
echo " Para instalarlo en Ubuntu/Debian ejecuta:"
echo "     sudo apt install ./$DEB_NAME"
echo "===================================================="
