#!/usr/bin/env bash
# ==============================================================================
# WolfDesk Linux Universal Installer & Uninstaller
# Compatible con: Ubuntu, Debian, Linux Mint, Fedora, CentOS, RHEL, Arch, Manjaro
# ==============================================================================

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

INSTALL_DIR="/opt/wolfdesk"
BIN_LINK="/usr/local/bin/wolfdesk"
DESKTOP_FILE="/usr/share/applications/wolfdesk.desktop"
ICON_DEST="/usr/share/icons/hicolor/256x256/apps/wolfdesk.png"
ID_DIR="/etc/wolfdesk"

echo -e "${BLUE}====================================================${NC}"
echo -e "${BLUE}        🐺 WolfDesk Remote Desktop - Linux          ${NC}"
echo -e "${BLUE}====================================================${NC}"

# Verificar permisos de superusuario
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}[!] Por favor, ejecuta este instalador con sudo o permisos de root:${NC}"
    echo -e "    sudo bash $0 $@"
    exit 1
fi

# Opción de desinstalación
if [ "$1" == "--uninstall" ] || [ "$1" == "-u" ]; then
    echo -e "${YELLOW}[*] Desinstalando WolfDesk del sistema...${NC}"
    rm -rf "$INSTALL_DIR"
    rm -f "$BIN_LINK"
    rm -f "$DESKTOP_FILE"
    rm -f "$ICON_DEST"
    rm -rf "$ID_DIR"

    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database /usr/share/applications || true
    fi
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
    fi

    echo -e "${GREEN}[✓] WolfDesk ha sido desinstalado completamente de su sistema.${NC}"
    exit 0
fi

# Detectar origen del binario
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PARENT_DIR="$(dirname "$SCRIPT_DIR")"

SOURCE_BIN=""
if [ -f "$SCRIPT_DIR/wolfdesk" ]; then
    SOURCE_BIN="$SCRIPT_DIR/wolfdesk"
elif [ -f "$PARENT_DIR/target/release/wolfdesk" ]; then
    SOURCE_BIN="$PARENT_DIR/target/release/wolfdesk"
elif [ -f "./wolfdesk" ]; then
    SOURCE_BIN="./wolfdesk"
fi

if [ -z "$SOURCE_BIN" ] || [ ! -f "$SOURCE_BIN" ]; then
    echo -e "${RED}[!] No se encontró el binario compilado 'wolfdesk'.${NC}"
    echo -e "    Asegúrese de compilar primero con: ${YELLOW}cargo build --release${NC}"
    exit 1
fi

echo -e "${BLUE}[*] Instalando WolfDesk en ${INSTALL_DIR}...${NC}"

# 1. Crear directorios
mkdir -p "$INSTALL_DIR"
mkdir -p "/usr/local/bin"
mkdir -p "/usr/share/applications"
mkdir -p "/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$ID_DIR"
chmod 755 "$ID_DIR"

# 2. Copiar binario y dar permisos de ejecución
cp "$SOURCE_BIN" "$INSTALL_DIR/wolfdesk"
chmod 755 "$INSTALL_DIR/wolfdesk"
ln -sf "$INSTALL_DIR/wolfdesk" "$BIN_LINK"

# 3. Copiar icono
ICON_SRC=""
if [ -f "$SCRIPT_DIR/assets/wolfdesk.png" ]; then
    ICON_SRC="$SCRIPT_DIR/assets/wolfdesk.png"
elif [ -f "$PARENT_DIR/crates/client-core/assets/wolfdesk.png" ]; then
    ICON_SRC="$PARENT_DIR/crates/client-core/assets/wolfdesk.png"
elif [ -f "./assets/wolfdesk.png" ]; then
    ICON_SRC="./assets/wolfdesk.png"
fi

if [ -n "$ICON_SRC" ] && [ -f "$ICON_SRC" ]; then
    cp "$ICON_SRC" "$ICON_DEST"
    mkdir -p "$INSTALL_DIR/assets"
    cp "$ICON_SRC" "$INSTALL_DIR/assets/wolfdesk.png"
fi

# 4. Crear lanzador de escritorio
DESKTOP_SRC=""
if [ -f "$SCRIPT_DIR/assets/wolfdesk.desktop" ]; then
    DESKTOP_SRC="$SCRIPT_DIR/assets/wolfdesk.desktop"
elif [ -f "$PARENT_DIR/crates/client-core/assets/wolfdesk.desktop" ]; then
    DESKTOP_SRC="$PARENT_DIR/crates/client-core/assets/wolfdesk.desktop"
fi

if [ -n "$DESKTOP_SRC" ] && [ -f "$DESKTOP_SRC" ]; then
    cp "$DESKTOP_SRC" "$DESKTOP_FILE"
else
    cat <<EOF > "$DESKTOP_FILE"
[Desktop Entry]
Version=1.0
Type=Application
Name=WolfDesk Pro
GenericName=Remote Desktop
Comment=Escritorio Remoto Seguro y Ultrarrápido
Exec=$BIN_LINK %u
Icon=wolfdesk
Terminal=false
Categories=Network;RemoteAccess;Utility;
Keywords=remote;desktop;anydesk;teamviewer;vnc;rdp;wolfdesk;
StartupWMClass=wolfdesk
EOF
fi
chmod 644 "$DESKTOP_FILE"

# 5. Actualizar bases de datos de escritorio e iconos
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
fi

echo -e "${GREEN}====================================================${NC}"
echo -e "${GREEN} [✓] ¡WolfDesk se ha instalado exitosamente!        ${NC}"
echo -e "${GREEN}====================================================${NC}"
echo -e " - Ejecutable: ${YELLOW}$BIN_LINK${NC}"
echo -e " - Identidad fija en: ${YELLOW}$ID_DIR/identity.json${NC}"
echo -e " - Puedes iniciarlo desde tu menú de aplicaciones o escribiendo: ${YELLOW}wolfdesk${NC}"
echo -e " - Para desinstalar en el futuro: ${YELLOW}sudo bash $0 --uninstall${NC}"
