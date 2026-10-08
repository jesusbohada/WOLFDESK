# Guía de Instalación, Identidad Fija y Soporte Multiplataforma - WolfDesk Pro

WolfDesk Pro ha sido diseñado con arquitectura multiplataforma en **Rust**, permitiendo ser utilizado como un ejecutable portátil independiente o como software formalmente instalado en el sistema operativo.

---

## 1. Ocultación de la Terminal Externa y Terminal Integrada en la App
* **Sin ventana negra de CMD:** El ejecutable ha sido compilado con `#![windows_subsystem = "windows"]`. Al hacer doble clic en `wolfdesk.exe`, Windows **no abre ninguna consola de comandos externa**.
* **Pestaña `📟 Terminal` Integrada:** Todos los registros, diagnósticos de red, intentos de conexión y eventos del sistema se muestran en tiempo real en la nueva pestaña **Terminal** dentro de la propia interfaz de WolfDesk.
* **Herramientas de Terminal:**
  * Auto-desplazamiento en tiempo real con colores semánticos (Cyan = Info, Amarillo = Advertencia, Rojo = Error, Verde = Éxito).
  * Botones para **Limpiar registros**, **Copiar todo al portapapeles** o **Guardar log en archivo** (`wolfdesk.log`).
  * Consola interactiva para ejecutar diagnósticos rápidos (`status`, `id`, `ping`, `clear`, `help`).

---

## 2. Identificador Fijo y Permanente por Equipo (Ed25519)
* **¿Cómo funciona el ID fijo?**
  * En la primera ejecución o instalación, WolfDesk genera un par de claves criptográficas seguras **Ed25519** y deriva un identificador numérico único de 9 dígitos (estilo AnyDesk: `XXX XXX XXX`).
  * Esta identidad se guarda de forma persistente en:
    * **Windows:** `%ProgramData%\WolfDesk\identity.json` (o `%AppData%\WolfDesk\identity.json`).
    * **Linux:** `/etc/wolfdesk/identity.json` (o `~/.config/wolfdesk/identity.json`).
    * **macOS:** `/Library/Application Support/WolfDesk/identity.json`.
* **Persistencia total:**
  * Aunque el equipo se reinicie, la app se cierre o se actualice el ejecutable, **el ID del equipo NUNCA cambia**.
* **Renovación de ID al desinstalar:**
  * Al desinstalar WolfDesk, el archivo `identity.json` es eliminado automáticamente del sistema.
  * Si el usuario vuelve a instalar la aplicación en el futuro, se generará una identidad completamente nueva con un nuevo ID.

---

## 3. Instalación en Windows
WolfDesk ofrece 3 métodos de instalación oficiales:

### Método A: Instalación con 1 Clic desde la Interfaz Gráfica
1. Ejecuta `wolfdesk.exe`.
2. Verás en la esquina superior derecha y en Ajustes el botón **`🚀 Instalar WolfDesk`**.
3. Haz clic en él y pulsa **`✅ Instalar Ahora`**.
4. WolfDesk se copiará a los programas del sistema, creará accesos directos en el **Escritorio** y **Menú Inicio**, y se registrará en **Configuración > Aplicaciones instaladas de Windows**.

### Método B: Instalación Rápida por Lotes (Script)
* Haz doble clic en `instalar_wolfdesk.bat` (o ejecuta `wolfdesk.exe --install` en PowerShell/CMD).

### Método C: Desinstalación Completa
* Haz doble clic en `desinstalar_wolfdesk.bat` (o ejecuta `wolfdesk.exe --uninstall`, o desinstálalo directamente desde el panel de control / Configuración de Windows). Esto borrará los accesos directos, el registro y la identidad guardada.

---

## 4. Soporte Multiplataforma (Linux y macOS)

El núcleo de WolfDesk (`proto`, `client-core`, `server`) es código Rust puro con abstracciones nativas condicionales (`#[cfg]`).

### Para compilar y empaquetar en Linux:
1. Instalar dependencias de compilación:
   ```bash
   sudo apt update && sudo apt install -y build-essential libxcb1-dev libx11-dev libasound2-dev
   ```
2. Compilar binario de producción:
   ```bash
   cargo build --release -p client-core
   ```
3. Instalar en el sistema:
   ```bash
   sudo cp target/release/wolfdesk /usr/local/bin/wolfdesk
   sudo cp assets/wolfdesk.png /usr/share/pixmaps/wolfdesk.png
   # El archivo wolfdesk.desktop ya queda configurado para el menú de aplicaciones
   ```

### Para compilar y empaquetar en macOS:
1. Compilar binario:
   ```bash
   cargo build --release -p client-core
   ```
2. Crear bundle `WolfDesk.app`:
   ```bash
   mkdir -p WolfDesk.app/Contents/{MacOS,Resources}
   cp target/release/wolfdesk WolfDesk.app/Contents/MacOS/
   cp assets/wolfdesk.png WolfDesk.app/Contents/Resources/
   ```
