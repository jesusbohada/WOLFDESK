# 🔷 FarmaDesk Pro - Escritorio Remoto Farmanorte

[![CI/CD Multiplatform](https://github.com/actions/workflows/release.yml/badge.svg)](.github/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Brand: Farmanorte](https://img.shields.io/badge/Farmanorte-%23003399-blue)](https://github.com/jesusbohada/FARMADESK)

**FarmaDesk Pro** es la solución corporativa de soporte y escritorio remoto de alto rendimiento desarrollada exclusivamente para **Farmanorte**. Diseñada con arquitectura en **Rust** de latencia ultrabaja, cifrado asimétrico Ed25519 de extremo a extremo, visor interactivo con aceleración gráfica por hardware, transferencia bidireccional de archivos, soporte multimonitor y persistencia de identidad fija por equipo.

---

## 🎨 Identidad Visual Corporativa (Farmanorte)

* **Azul Corporativo Principal**: `#003399` (`RGB: 0, 51, 153` / `CMYK: 100, 80, 0, 10`)
* **Cyan Brillante de Acento**: `#00AFEF` (`RGB: 0, 175, 239` / `CMYK: 100, 0, 0, 0`)
* **Isotipo Oficial**: Emblema circular con el lazo "F" de Farmanorte integrado nativamente en iconos del sistema (.ico, .png) y en la barra de herramientas.

---

## ✨ Características Principales

- ⚡ **Latencia Ultrabaja**: Captura GDI acelerada y renderizado directo con adaptación fluida a 60 FPS.
- 🔒 **Cifrado E2E Asimétrico**: Autenticación criptográfica con llaves Ed25519. El ID del equipo es permanente y fijo.
- 🖥️ **Multiplataforma Nativo**: Compatible con **Windows** (10/11), **Linux** (Ubuntu, Debian, Fedora, Arch) y **macOS** (Apple Silicon M-series e Intel).
- 📁 **Gestor de Transferencia de Archivos**: Explorador bidireccional rápido entre el equipo local y el remoto con acceso directo al Home de usuario.
- 🎛️ **Barra de Herramientas Flotante**:
  - Escalabilidad de pantalla (Ajustar a ventana, Estirar 100%, Píxel 1:1 original).
  - Selector de calidad en tiempo real (F1 Económico, F2 Equilibrado, F3 Alta, F4 Máxima 96% Nativa).
  - Selector dinámico de pantalla remota (Monitor 1, Monitor 2, etc.).
  - Modo Espectador Silencioso (Solo lectura / Auditoría sin interrumpir al usuario remoto).
  - Grabación de sesión en vídeo/fotogramas.
  - Sincronización completa de teclado (incluyendo soporte integral para tecla Windows y combinaciones especiales).
- 📟 **Terminal Integrada en la App**: Monitor de logs, diagnóstico de latencia y comandos administrativos sin consolas externas.
- 🔄 **Auto-Actualizador Inteligente**: Descarga directa de ejecutables precompilados desde GitHub con un solo clic y reinicio en caliente.
- 🚀 **Instalable en 1 Clic**: Modo portable o instalación completa en el sistema con accesos directos y desinstalador automático.

---

## 🏗️ Estructura del Proyecto

```
farmadesk/
├── .github/workflows/         # Automatización CI/CD Multiplataforma (GitHub Actions)
│   └── release.yml
├── crates/
│   ├── proto/                 # Protocolo de señalización, mensajes y criptografía Ed25519
│   ├── server/                # Servidor de señalización y retransmisión WebSocket (Rendezvous/Relay)
│   └── client-core/           # Aplicación cliente GUI (Dashboard egui + Visor minifb + BrandConfig)
├── scripts/
│   ├── install_linux.sh       # Instalador universal para Linux con integración de escritorio
│   ├── package_deb.sh         # Generador de paquetes Debian (.deb) para Ubuntu/Debian/Mint
│   └── package_macos.sh       # Generador de bundle .app y .dmg para macOS
├── Cargo.toml                 # Workspace Cargo
├── farmadesk.exe              # Ejecutable precompilado oficial para Windows
├── reiniciar_farmadesk.bat    # Script de reinicio rápido en 1 clic para Windows
└── GUIA_COMPLETA_LINUX_Y_MACOS.md # Documentación para Linux y macOS
```

---

## 🚀 Compilación y Ejecución

### Requisitos previos
- [Rust & Cargo](https://rustup.rs/) (1.75 o superior)

### 1. Iniciar el Servidor de Señalización
```bash
cargo run --release --bin server
```

### 2. Iniciar el Cliente FarmaDesk
```bash
cargo run --release --bin farmadesk --features farmadesk
```

---

## 📦 Instalación en el Sistema

### 🪟 Windows (10 / 11)
1. Descarga el ejecutable oficial `farmadesk.exe` desde la raíz de este repositorio o compílalo con:
   ```bash
   cargo build --release --bin farmadesk --features farmadesk
   ```
2. Para instalar permanentemente en el sistema con accesos directos en el Escritorio e Inicio:
   - Haz clic en el botón **"🚀 Instalar FarmaDesk"** en la pestaña Inicio o en Ajustes.
   - O bien ejecuta el asistente rápido `reiniciar_farmadesk.bat` para desbloquear el proceso si requieres mantenimiento.

---

### 🐧 Linux (Ubuntu / Debian / Linux Mint / Fedora / Arch)

Abre la terminal en tu equipo Linux y ejecuta los siguientes pasos:

#### Paso 1: Instalar dependencias y herramientas
```bash
sudo apt update || true
sudo apt install -y git build-essential pkg-config libx11-dev libasound2-dev libxcursor-dev libxrandr-dev libxi-dev libgl1-mesa-dev libxkbcommon-dev curl
```

#### Paso 2: Instalar Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

#### Paso 3: Descargar el proyecto y entrar a la carpeta
```bash
git clone https://github.com/jesusbohada/FARMADESK.git
cd FARMADESK
```

#### Paso 4: Compilar e Instalar FarmaDesk
```bash
cargo build --release --bin farmadesk --features farmadesk
sudo bash scripts/install_linux.sh
```
> ✅ **¡Listo!** FarmaDesk quedará instalado en `/opt/farmadesk`, con acceso directo en tu menú de aplicaciones y comando global `farmadesk`.

*(Opcional: Si prefieres empaquetar un archivo `.deb` para instalar con doble clic en cualquier máquina Debian/Ubuntu):*
```bash
bash scripts/package_deb.sh farmadesk
sudo apt install ./farmadesk_1.2.0_amd64.deb
```

---

### 🍎 macOS (Apple Silicon M-series e Intel)

En una Mac con macOS Monterey, Ventura, Sonoma o Sequoia:

```bash
# 1. Herramientas y Rust
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# 2. Descargar el proyecto
git clone https://github.com/jesusbohada/FARMADESK.git
cd FARMADESK

# 3. Compilar y empaquetar aplicación (.app y .dmg)
bash scripts/package_macos.sh farmadesk
```
> *(Se generará el archivo `FarmaDesk.app` para arrastrar a `/Applications` y la imagen de disco `FarmaDesk-macOS-Universal.dmg`).*

---

## 📄 Licencia
Este proyecto está bajo la Licencia MIT. Consulta el archivo `LICENSE` para más detalles.

