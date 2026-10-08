# 🐺 WolfDesk Pro - High-Performance Remote Desktop

[![CI/CD Multiplatform](https://github.com/actions/workflows/release.yml/badge.svg)](.github/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)]()

**WolfDesk Pro** es una solución de escritorio remoto ultrarrápida, segura y auto-hospedada desarrollada en **Rust**. Diseñada con arquitectura de latencia ultrabaja, cifrado asimétrico Ed25519 de extremo a extremo, visor interactivo con aceleración por hardware, transferencia bidireccional de archivos y persistencia de identidad fija por equipo.

---

## ✨ Características Principales

- ⚡ **Latencia Ultrabaja**: Captura GDI acelerada y renderizado directo con adaptación fluida a 60 FPS.
- 🔒 **Cifrado E2E Asimétrico**: Autenticación criptográfica con llaves Ed25519. El ID del equipo es permanente y fijo.
- 🖥️ **Multiplataforma Nativo**: Compatible con **Windows** (10/11), **Linux** (Ubuntu, Debian, Fedora, Arch) y **macOS** (Apple Silicon M-series e Intel).
- 📁 **Gestor de Transferencia de Archivos**: Explorador bidireccional tipo AnyDesk entre el equipo local y el remoto.
- 🎛️ **Barra de Herramientas Flotante**:
  - Escalabilidad de pantalla (Ajustar a ventana, Estirar 100%, Píxel 1:1 original).
  - Selector de calidad de imagen en tiempo real (F1 Económico, F2 Estándar, F3 Alta, F4 Máxima 96% Nativa).
  - Grabación de sesión en vídeo/fotogramas.
  - Sincronización de teclado (incluyendo soporte completo para tecla Windows y combinaciones especiales).
- 📟 **Terminal Integrada en la App**: Monitor de logs y comandos administrativos sin ventanas de consola negras externas.
- 🚀 **Instalable en 1 Clic**: Modo portable o instalación completa en el sistema operativo con accesos directos y desinstalador automático.

---

## 🏗️ Estructura del Proyecto

```
remote-desktop/
├── .github/workflows/         # Automatización CI/CD Multiplataforma (GitHub Actions)
│   └── release.yml
├── crates/
│   ├── proto/                 # Protocolo de señalización, mensajes y criptografía Ed25519
│   ├── server/                # Servidor de señalización y retransmisión WebSocket (Rendezvous/Relay)
│   └── client-core/           # Aplicación cliente GUI (Dashboard egui + Visor minifb)
├── scripts/
│   ├── install_linux.sh       # Instalador universal para Linux con integración de escritorio
│   ├── package_deb.sh         # Generador de paquetes Debian (.deb) para Ubuntu/Debian/Mint
│   └── package_macos.sh       # Generador de bundle .app y .dmg para macOS
├── Cargo.toml                 # Workspace Cargo
├── wolfdesk_setup.iss         # Script Inno Setup para generar instalador clásico de Windows
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

### 2. Iniciar el Cliente WolfDesk
```bash
cargo run --release --bin wolfdesk
```

---

## 📦 Instalación en el Sistema

### 🪟 Windows (10 / 11)
1. Descarga el ejecutable `wolfdesk.exe` o compila con `cargo build --release --bin wolfdesk`.
2. Para instalar permanentemente en el sistema con accesos directos en el Escritorio e Inicio:
   - Haz clic en el botón **"🚀 Instalar WolfDesk"** en la pestaña Inicio o en Ajustes.
   - O bien ejecuta el asistente rápido `instalar_wolfdesk.bat`.
   - O genera el instalador clásico `.exe` con el script Inno Setup `wolfdesk_setup.iss`.

---

### 🐧 Linux (Ubuntu / Debian / Linux Mint / Fedora / Arch)

Abre la terminal en tu equipo Linux y ejecuta estos 3 pasos:

#### 1. Instalar dependencias del sistema y Rust (Automático)
```bash
sudo apt update && sudo apt install -y git build-essential pkg-config libx11-dev libasound2-dev libxcursor-dev libxrandr-dev libxi-dev libgl1-mesa-dev libxkbcommon-dev curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

#### 2. Descargar el proyecto y entrar a la carpeta
```bash
git clone https://github.com/jesusbohada/WOLFDESK.git
cd WOLFDESK
```

#### 3. Compilar e Instalar
```bash
cargo build --release --bin wolfdesk
sudo bash scripts/install_linux.sh
```
> *(WolfDesk quedará instalado en `/opt/wolfdesk`, con acceso directo en tu menú de aplicaciones y ejecutable global `wolfdesk`).*

*(Opcional: Si deseas generar un instalador `.deb` para instalar con doble clic):*
```bash
bash scripts/package_deb.sh
sudo apt install ./wolfdesk_1.2.0_amd64.deb
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
git clone https://github.com/jesusbohada/WOLFDESK.git
cd WOLFDESK

# 3. Compilar y empaquetar aplicación (.app y .dmg)
bash scripts/package_macos.sh
```
> *(Se generará el archivo `WolfDesk.app` para arrastrar a `/Applications` y la imagen de disco `WolfDesk-macOS-Universal.dmg`).*

---

## 📄 Licencia
Este proyecto está bajo la Licencia MIT. Consulta el archivo `LICENSE` para más detalles.
