# 🐺 Guía Completa de WolfDesk para Linux y macOS

Esta guía detalla con precisión cómo compilar, empaquetar, distribuir e instalar **WolfDesk** en distribuciones **Linux** (Ubuntu, Debian, Fedora, Arch, Linux Mint) y en **macOS** (Apple Silicon M1/M2/M3/M4 y procesadores Intel).

---

## 📑 Índice
1. [¿Cómo funciona la compatibilidad multiplataforma de WolfDesk?](#1-cómo-funciona-la-compatibilidad-multiplataforma-de-wolfdesk)
2. [Método 1: Compilación Automatizada en la Nube con GitHub Actions (Recomendado)](#2-método-1-compilación-automatizada-en-la-nube-con-github-actions-recomendado)
3. [Método 2: Compilación e Instalación Nativa en Linux](#3-método-2-compilación-e-instalación-nativa-en-linux)
4. [Método 3: Compilación e Instalación Nativa en macOS](#4-método-3-compilación-e-instalación-nativa-en-macos)
5. [Estructura del ID Fijo Permanente en cada Sistema Operativo](#5-estructura-del-id-fijo-permanente-en-cada-sistema-operativo)
6. [Permisos y Servidores Gráficos (X11, Wayland y macOS Security)](#6-permisos-y-servidores-gráficos-x11-wayland-y-macos-security)

---

## 1. ¿Cómo funciona la compatibilidad multiplataforma de WolfDesk?

WolfDesk está desarrollado en **Rust** utilizando bibliotecas multiplataforma nativas:
- **GUI Dashboard**: `egui` y `eframe` (se integran con OpenGL, Vulkan, Metal o DirectX según el SO).
- **Visor Remoto ultrarrápido**: `minifb` (soporta X11 y Wayland en Linux, Cocoa/Metal en macOS, Win32 en Windows).
- **Red y Criptografía**: `tokio` (WebSockets asíncronos) y `ed25519-dalek` (cifrado asimétrico E2E).
- **Instalación y Desinstalación**:
  - En **Windows**: Crea accesos directos `.lnk` en Escritorio/Inicio y se registra en `Configuración > Aplicaciones`.
  - En **Linux**: Se instala en `/opt/wolfdesk/`, crea enlace simbólico en `/usr/local/bin/wolfdesk` y genera el archivo estándar `/usr/share/applications/wolfdesk.desktop` con su icono.
  - En **macOS**: Se empaqueta como aplicación estándar `WolfDesk.app` dentro de `/Applications/`.

---

## 2. Método 1: Compilación Automatizada en la Nube con GitHub Actions (Recomendado)

Si tu equipo principal es Windows, **no necesitas instalar máquinas virtuales ni comprar una Mac** para generar los instaladores de Linux y macOS.

WolfDesk incluye el flujo de trabajo `.github/workflows/release.yml`.

### ¿Cómo activarlo?
1. Sube tu repositorio a GitHub.
2. Cada vez que crees una versión o etiqueta (tag), por ejemplo:
   ```bash
   git tag v1.2.0
   git push origin v1.2.0
   ```
3. GitHub lanzará en paralelo 3 servidores reales de compilación:
   - **Servidor Windows**: Genera `WolfDesk-Windows-x64.zip` y `wolfdesk.exe`.
   - **Servidor Ubuntu Linux**: Genera `wolfdesk_1.2.0_amd64.deb` (para Ubuntu/Debian) y `WolfDesk-Linux-x86_64.tar.gz`.
   - **Servidor Apple macOS**: Genera el binario universal (M1/M2/M3 + Intel) y el instalador `WolfDesk-macOS-Universal.dmg`.
4. Todos los archivos quedan listos para descarga pública o privada en la pestaña **Releases** de tu GitHub.

---

## 3. Método 2: Compilación e Instalación Nativa en Linux

Si tienes un equipo con Linux (o estás conectado por SSH), puedes compilar e instalar en cuestión de segundos:

### Paso 1: Instalar dependencias del sistema

#### En Ubuntu / Debian / Linux Mint:
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libx11-dev libasound2-dev libxcursor-dev libxrandr-dev libxi-dev libgl1-mesa-dev libxkbcommon-dev curl git
```

#### En Fedora / Red Hat / CentOS:
```bash
sudo dnf install -y gcc gcc-c++ make pkgconf-pkg-config libX11-devel alsa-lib-devel libXcursor-devel libXrandr-devel libXi-devel mesa-libGL-devel libxkbcommon-devel curl git
```

#### En Arch Linux / Manjaro:
```bash
sudo pacman -S --needed base-devel alsa-lib libx11 libxcursor libxrandr libxi libxkbcommon rustup git
```

### Paso 2: Instalar Rust (de forma automática)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### Paso 3: Descargar y Compilar WolfDesk
```bash
git clone https://github.com/jesusbohada/WOLFDESK.git
cd WOLFDESK
cargo build --release --bin wolfdesk
```

### Paso 4: Instalar en el sistema
Tienes dos alternativas para instalar:

#### Alternativa A: Usar el instalador universal incluido
```bash
sudo bash scripts/install_linux.sh
```
Esto colocará el ejecutable en `/opt/wolfdesk/wolfdesk`, creará el acceso directo en el menú de aplicaciones y configurará la carpeta del ID fijo.

#### Alternativa B: Generar e instalar el paquete Debian (.deb)
```bash
bash scripts/package_deb.sh
sudo apt install ./wolfdesk_1.2.0_amd64.deb
```

#### Para desinstalar en Linux:
```bash
sudo bash scripts/install_linux.sh --uninstall
# O si usaste el .deb:
sudo apt remove wolfdesk
```

---

## 4. Método 3: Compilación e Instalación Nativa en macOS

### Paso 1: Instalar herramientas de compilación
Abre la Terminal de tu Mac y ejecuta:
```bash
xcode-select --install
```

### Paso 2: Instalar Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### Paso 3: Descargar y Compilar WolfDesk
```bash
git clone https://github.com/jesusbohada/WOLFDESK.git
cd WOLFDESK
cargo build --release --bin wolfdesk
```

### Paso 4: Empaquetar como aplicación macOS (.app y .dmg)
Ejecuta el script de empaquetado incluido:
```bash
bash scripts/package_macos.sh
```
Esto generará:
1. `WolfDesk.app` en la carpeta `dist_macos/` (puedes arrastrarlo directamente a tu carpeta `/Applications`).
2. `WolfDesk-macOS-Universal.dmg` (instalador tradicional arrastrable de macOS).

---

## 5. Estructura del ID Fijo Permanente en cada Sistema Operativo

El ID de 9 dígitos de WolfDesk se deriva criptográficamente de un par de llaves Ed25519 permanente. Las rutas donde se aloja son:

| Sistema Operativo | Ruta de Almacenamiento Global | Ruta de Respaldo por Usuario |
|---|---|---|
| **Windows** | `C:\ProgramData\WolfDesk\identity.json` | `%APPDATA%\WolfDesk\identity.json` |
| **Linux** | `/etc/wolfdesk/identity.json` | `~/.config/wolfdesk/identity.json` |
| **macOS** | `/Library/Application Support/WolfDesk/identity.json` | `~/Library/Application Support/WolfDesk/identity.json` |

> 🔒 **Garantía de Persistencia**: Aunque se reinicie el equipo, se apague o se actualice la aplicación a una versión nueva, el ID se mantendrá intacto. Solo se elimina y regenera si se ejecuta la acción de **Desinstalar**.

---

## 6. Permisos y Servidores Gráficos (X11, Wayland y macOS Security)

### En Linux (X11 vs Wayland):
- **Modo Cliente (Conectarse a otro equipo)**: Funciona de forma 100% transparente tanto en X11 como en Wayland.
- **Modo Servidor / Host (Compartir pantalla de Linux)**:
  - En sesiones **X11** (predeterminadas en muchas distribuciones y escritorios XFCE/KDE), la captura de pantalla y el control remoto son directos y sin restricciones.
  - En sesiones **Wayland** (Ubuntu moderno por defecto), por motivos de aislamiento de seguridad de Wayland, el sistema solicitará autorización para compartir la pantalla mediante PipeWire / XDG Desktop Portal.

### En macOS:
Apple exige permisos explícitos de privacidad para aplicaciones de acceso remoto:
1. **Grabación de pantalla**: Ir a `Ajustes del Sistema > Privacidad y Seguridad > Grabación de Pantalla` y activar **WolfDesk**.
2. **Accesibilidad**: Ir a `Ajustes del Sistema > Privacidad y Seguridad > Accesibilidad` y activar **WolfDesk** (necesario para permitir que el mouse y teclado remotos muevan el cursor).
