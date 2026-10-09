#!/usr/bin/env bash
# ==============================================================================
# WolfDesk Pro - Script Universal de Actualización
# Actualiza el código fuente desde GitHub, compila y despliega el binario
# ==============================================================================

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}====================================================${NC}"
echo -e "${BLUE}        🐺 WolfDesk - Actualizador del Sistema       ${NC}"
echo -e "${BLUE}====================================================${NC}"

# 1. Localizar el directorio del repositorio
REPO_DIR=""
if [ -d "/home/caja/WOLFDESK/.git" ]; then
    REPO_DIR="/home/caja/WOLFDESK"
elif [ -d "$PWD/.git" ]; then
    REPO_DIR="$PWD"
else
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    if [ -d "$SCRIPT_DIR/../.git" ]; then
        REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
    fi
fi

if [ -z "$REPO_DIR" ] || [ ! -d "$REPO_DIR" ]; then
    echo -e "${RED}[!] No se pudo localizar el directorio del repositorio WOLFDESK.${NC}"
    exit 1
fi

echo -e "${BLUE}[*] Directorio de trabajo:${NC} ${YELLOW}$REPO_DIR${NC}"
cd "$REPO_DIR"

# 2. Descargar los últimos cambios desde GitHub
echo -e "${BLUE}[*] Descargando últimos cambios desde GitHub (git pull origin main)...${NC}"
git fetch origin main
git checkout main
git pull origin main

# 3. Compilar el binario optimizado
echo -e "${BLUE}[*] Compilando la versión más reciente con Cargo (Release)...${NC}"
cargo build --release --bin wolfdesk

NEW_BIN="$REPO_DIR/target/release/wolfdesk"
if [ ! -f "$NEW_BIN" ]; then
    echo -e "${RED}[!] Error: No se encontró el binario compilado en $NEW_BIN.${NC}"
    exit 1
fi

# 4. Actualizar la instalación en /opt/wolfdesk si existe
if [ -d "/opt/wolfdesk" ]; then
    echo -e "${BLUE}[*] Actualizando binario en /opt/wolfdesk/wolfdesk...${NC}"
    cp -f "$NEW_BIN" /opt/wolfdesk/wolfdesk
    chmod 755 /opt/wolfdesk/wolfdesk
    if [ -f "$REPO_DIR/crates/client-core/assets/wolfdesk.png" ]; then
        mkdir -p /opt/wolfdesk/assets
        cp -f "$REPO_DIR/crates/client-core/assets/wolfdesk.png" /opt/wolfdesk/assets/wolfdesk.png
    fi
fi

# 5. Si el servicio systemd existe, recargar
if [ -f "/etc/systemd/system/wolfdesk.service" ]; then
    echo -e "${BLUE}[*] Recargando configuración de servicios systemd...${NC}"
    systemctl daemon-reload || true
fi

COMMIT_HASH=$(git rev-parse --short HEAD)
echo -e "${GREEN}====================================================${NC}"
echo -e "${GREEN} [✓] ¡WolfDesk actualizado exitosamente a [$COMMIT_HASH]! ${NC}"
echo -e "${GREEN}====================================================${NC}"
echo -e "Cerrando instancias anteriores para aplicar los cambios..."
pkill -9 -f /opt/wolfdesk/wolfdesk 2>/dev/null || true
pkill -9 -f target/release/wolfdesk 2>/dev/null || true
sleep 1

if [ -f "/etc/systemd/system/wolfdesk.service" ] && systemctl is-active --quiet wolfdesk; then
    echo -e "${BLUE}[*] Reiniciando servicio wolfdesk.service...${NC}"
    systemctl restart wolfdesk
fi

echo -e "${GREEN}[✓] Todo listo. Puedes iniciar WolfDesk ahora con:${NC} ${YELLOW}wolfdesk${NC} o ${YELLOW}./target/release/wolfdesk${NC}"
