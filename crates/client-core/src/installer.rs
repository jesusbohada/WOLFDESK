use std::fs;
use std::path::{Path, PathBuf};

/// Comprueba si WolfDesk ya está instalado formalmente en el sistema
pub fn is_installed() -> bool {
    if let Ok(current) = std::env::current_exe() {
        let path_str = current.to_string_lossy().to_lowercase();
        #[cfg(windows)]
        {
            if path_str.contains("program files\\wolfdesk")
                || path_str.contains("programs\\wolfdesk")
            {
                return true;
            }
        }
        #[cfg(target_os = "linux")]
        {
            if path_str.starts_with("/usr/") || path_str.starts_with("/opt/wolfdesk") {
                return true;
            }
        }
        #[cfg(target_os = "macos")]
        {
            if path_str.contains("/applications/wolfdesk.app") {
                return true;
            }
        }
    }
    false
}

/// Obtiene el directorio de instalación óptimo según el sistema y permisos
pub fn get_install_dir() -> PathBuf {
    #[cfg(windows)]
    {
        // 1. Si hay acceso a Program Files (administrador), preferimos instalación global
        if let Ok(prog_files) = std::env::var("ProgramFiles") {
            let target = PathBuf::from(prog_files).join("WolfDesk");
            if fs::create_dir_all(&target).is_ok() {
                return target;
            }
        }
        // 2. Si no, instalamos en el directorio de programas locales del usuario (sin requerir UAC/Admin)
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            return PathBuf::from(local_appdata).join("Programs").join("WolfDesk");
        }
        PathBuf::from("C:\\WolfDesk")
    }

    #[cfg(target_os = "linux")]
    {
        PathBuf::from("/opt/wolfdesk")
    }

    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/Applications/WolfDesk.app/Contents/MacOS")
    }
}

/// Ejecuta la instalación completa de WolfDesk en el sistema operativo
pub fn install_to_system() -> Result<String, String> {
    let current_exe = std::env::current_exe().map_err(|e| format!("Error al obtener ejecutable actual: {}", e))?;
    let install_dir = get_install_dir();

    fs::create_dir_all(&install_dir).map_err(|e| format!("Error al crear carpeta de instalación {:?}: {}", install_dir, e))?;

    let exe_name = if cfg!(windows) { "wolfdesk.exe" } else { "wolfdesk" };
    let dest_exe = install_dir.join(exe_name);

    // Copiar el ejecutable a la carpeta de destino
    if current_exe != dest_exe {
        fs::copy(&current_exe, &dest_exe).map_err(|e| format!("Error al copiar ejecutable a {:?}: {}", dest_exe, e))?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(&dest_exe) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(&dest_exe, perms);
        }
    }

    // Copiar assets si existen
    if Path::new("assets/wolfdesk.png").exists() {
        let dest_assets = install_dir.join("assets");
        let _ = fs::create_dir_all(&dest_assets);
        let _ = fs::copy("assets/wolfdesk.png", dest_assets.join("wolfdesk.png"));
    }

    #[cfg(windows)]
    {
        create_windows_shortcuts(&dest_exe)?;
        register_windows_uninstaller(&dest_exe, &install_dir)?;
    }

    #[cfg(target_os = "linux")]
    {
        create_linux_desktop_entry(&dest_exe)?;
    }

    log::info!("✅ WolfDesk instalado exitosamente en: {:?}", install_dir);
    Ok(format!("WolfDesk se ha instalado exitosamente en: {}", install_dir.display()))
}

/// Desinstala WolfDesk del sistema, eliminando accesos directos, registro y la identidad fija
pub fn uninstall_from_system() -> Result<String, String> {
    #[cfg(windows)]
    {
        remove_windows_shortcuts()?;
        remove_windows_uninstaller()?;
    }

    #[cfg(target_os = "linux")]
    {
        let _ = fs::remove_file("/usr/share/applications/wolfdesk.desktop");
        if let Ok(home) = std::env::var("HOME") {
            let user_desktop = PathBuf::from(home).join(".local/share/applications/wolfdesk.desktop");
            let _ = fs::remove_file(user_desktop);
        }
    }

    // Eliminar la identidad persistente para que en una futura reinstalación se genere un ID nuevo
    crate::identity_store::remove_identity_on_uninstall();

    log::info!("🗑️ WolfDesk ha sido desinstalado del sistema.");
    Ok("WolfDesk ha sido desinstalado correctamente del equipo.".to_string())
}

#[cfg(windows)]
fn create_windows_shortcuts(dest_exe: &Path) -> Result<(), String> {
    use std::process::Command;

    let exe_path_str = dest_exe.to_string_lossy().to_string();

    let script = format!(
        r#"$WshShell = New-Object -ComObject WScript.Shell
$DesktopPath = [Environment]::GetFolderPath('Desktop')
$Shortcut = $WshShell.CreateShortcut("$DesktopPath\WolfDesk.lnk")
$Shortcut.TargetPath = "{}"
$Shortcut.IconLocation = "{},0"
$Shortcut.Description = "WolfDesk Pro - Escritorio Remoto Seguro"
$Shortcut.Save()

$StartMenuPath = [Environment]::GetFolderPath('Programs')
$Shortcut2 = $WshShell.CreateShortcut("$StartMenuPath\WolfDesk.lnk")
$Shortcut2.TargetPath = "{}"
$Shortcut2.IconLocation = "{},0"
$Shortcut2.Description = "WolfDesk Pro - Escritorio Remoto Seguro"
$Shortcut2.Save()"#,
        exe_path_str, exe_path_str, exe_path_str, exe_path_str
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .output()
        .map_err(|e| format!("Error al ejecutar PowerShell para accesos directos: {}", e))?;

    if !output.status.success() {
        log::warn!("Advertencia al crear accesos directos: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

#[cfg(windows)]
fn remove_windows_shortcuts() -> Result<(), String> {
    use std::process::Command;

    let script = r#"$DesktopPath = [Environment]::GetFolderPath('Desktop')
if (Test-Path "$DesktopPath\WolfDesk.lnk") { Remove-Item "$DesktopPath\WolfDesk.lnk" -Force }
$StartMenuPath = [Environment]::GetFolderPath('Programs')
if (Test-Path "$StartMenuPath\WolfDesk.lnk") { Remove-Item "$StartMenuPath\WolfDesk.lnk" -Force }
"#;

    let _ = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
        .output();

    Ok(())
}

#[cfg(windows)]
fn register_windows_uninstaller(dest_exe: &Path, install_dir: &Path) -> Result<(), String> {
    use std::process::Command;

    let exe_str = dest_exe.to_string_lossy().to_string();
    let dir_str = install_dir.to_string_lossy().to_string();

    let script = format!(
        r#"$regPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\WolfDesk"
if (!(Test-Path $regPath)) {{ New-Item -Path $regPath -Force | Out-Null }}
Set-ItemProperty -Path $regPath -Name "DisplayName" -Value "WolfDesk Pro - Escritorio Remoto"
Set-ItemProperty -Path $regPath -Name "DisplayVersion" -Value "1.2.0"
Set-ItemProperty -Path $regPath -Name "Publisher" -Value "WolfDesk Software"
Set-ItemProperty -Path $regPath -Name "DisplayIcon" -Value "{},0"
Set-ItemProperty -Path $regPath -Name "InstallLocation" -Value "{}"
Set-ItemProperty -Path $regPath -Name "UninstallString" -Value '"{}" --uninstall'
Set-ItemProperty -Path $regPath -Name "NoModify" -Value 1 -Type DWord
Set-ItemProperty -Path $regPath -Name "NoRepair" -Value 1 -Type DWord
"#,
        exe_str, dir_str, exe_str
    );

    let _ = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .output();

    Ok(())
}

#[cfg(windows)]
fn remove_windows_uninstaller() -> Result<(), String> {
    use std::process::Command;

    let script = r#"$regPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\WolfDesk"
if (Test-Path $regPath) { Remove-Item -Path $regPath -Recurse -Force }
"#;

    let _ = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
        .output();

    Ok(())
}

#[cfg(target_os = "linux")]
fn create_linux_desktop_entry(dest_exe: &Path) -> Result<(), String> {
    let entry = format!(
        "[Desktop Entry]\n\
        Name=WolfDesk\n\
        Comment=WolfDesk Remote Desktop\n\
        Exec={}\n\
        Icon=wolfdesk\n\
        Terminal=false\n\
        Type=Application\n\
        Categories=Network;RemoteAccess;\n",
        dest_exe.display()
    );
    let _ = fs::write("/usr/share/applications/wolfdesk.desktop", &entry);
    if let Ok(home) = std::env::var("HOME") {
        let user_apps = PathBuf::from(home).join(".local/share/applications");
        let _ = fs::create_dir_all(&user_apps);
        let _ = fs::write(user_apps.join("wolfdesk.desktop"), &entry);
    }
    Ok(())
}
