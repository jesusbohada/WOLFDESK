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

# 1. Asegurar entorno PATH completo para Cargo, Rustup y utilidades del sistema
for env_file in "$HOME/.cargo/env" "/home/caja/.cargo/env" "/root/.cargo/env"; do
    if [ -f "$env_file" ]; then
        source "$env_file" || true
        break
    fi
done
export PATH="$HOME/.cargo/bin:/home/caja/.cargo/bin:/root/.cargo/bin:/usr/local/cargo/bin:/usr/local/bin:/usr/bin:/bin:$PATH"

# 2. Localizar el directorio del repositorio
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

# 3. Descargar los últimos cambios desde GitHub
echo -e "${BLUE}[*] Descargando últimos cambios desde GitHub (git fetch y git reset)...${NC}"
git fetch origin main
git checkout -f main
git reset --hard origin/main

# 4. Compilar el binario optimizado
echo -e "${BLUE}[*] Compilando la versión más reciente con Cargo (Release)...${NC}"
touch "$REPO_DIR/crates/client-core/build.rs" 2>/dev/null || true
COMMIT_HASH=$(git rev-parse --short HEAD 2>/dev/null || echo "10eb531")
export WOLFDESK_BUILD_GIT_HASH="$COMMIT_HASH"
cargo build --release --bin wolfdesk

NEW_BIN="$REPO_DIR/target/release/wolfdesk"
if [ ! -f "$NEW_BIN" ]; then
    echo -e "${RED}[!] Error: No se encontró el binario compilado en $NEW_BIN.${NC}"
    exit 1
fi

# 5. Actualizar la instalación en /opt/wolfdesk si existe
if [ -d "/opt/wolfdesk" ]; then
    echo -e "${BLUE}[*] Actualizando binario en /opt/wolfdesk/wolfdesk...${NC}"
    rm -f /opt/wolfdesk/wolfdesk 2>/dev/null || true
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

echo -e "${GREEN}====================================================${NC}"
echo -e "${GREEN} [✓] ¡WolfDesk actualizado exitosamente a [$COMMIT_HASH]! ${NC}"
echo -e "${GREEN}====================================================${NC}"

if [ -z "$WOLFDESK_IN_APP_UPDATE" ]; then
    echo -e "Cerrando instancias anteriores para aplicar los cambios..."
    pkill -9 -f /opt/wolfdesk/wolfdesk 2>/dev/null || true
    pkill -9 -f target/release/wolfdesk 2>/dev/null || true
    sleep 1

    if [ -f "/etc/systemd/system/wolfdesk.service" ] && systemctl is-active --quiet wolfdesk; then
        echo -e "${BLUE}[*] Reiniciando servicio wolfdesk.service...${NC}"
        systemctl restart wolfdesk
    fi
fi

echo -e "${GREEN}[✓] Todo listo. Puedes iniciar WolfDesk ahora con:${NC} ${YELLOW}wolfdesk${NC} o ${YELLOW}./target/release/wolfdesk${NC}"
