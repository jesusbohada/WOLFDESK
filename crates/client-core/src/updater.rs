#[allow(unused_imports)]
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn get_local_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn get_build_git_hash() -> &'static str {
    const COMPILED_HASH: &str = include_str!(concat!(env!("OUT_DIR"), "/build_git_hash.txt"));
    COMPILED_HASH.trim()
}

#[derive(Clone, Debug, PartialEq)]
pub enum UpdateStatus {
    NotChecked,
    Checking,
    UpToDate {
        commit: String,
        checked_time: String,
    },
    UpdateAvailable {
        current_commit: String,
        latest_commit: String,
        commit_message: String,
        date: String,
    },
    Updating {
        step: String,
    },
    Success {
        message: String,
    },
    Error {
        message: String,
    },
}

#[derive(Clone, Debug)]
pub struct RemoteUpdateInfo {
    pub commit_hash: String,
    pub short_hash: String,
    pub message: String,
    pub date: String,
}

/// Construye un PATH enriquecido para encontrar Cargo, Rustup, Git y utilidades en Linux o Windows
pub fn get_augmented_path() -> String {
    let current_path = std::env::var("PATH").unwrap_or_default();
    let mut parts: Vec<String> = Vec::new();

    let candidate_dirs = [
        "/home/caja/.cargo/bin",
        "/root/.cargo/bin",
        "/usr/local/cargo/bin",
        "/usr/local/bin",
        "/usr/bin",
        "/bin",
    ];

    for dir in &candidate_dirs {
        if Path::new(dir).exists() {
            parts.push(dir.to_string());
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home).join(".cargo/bin");
        if p.exists() {
            parts.push(p.to_string_lossy().to_string());
        }
    }

    parts.push(current_path);
    parts.join(if cfg!(windows) { ";" } else { ":" })
}

/// Localiza de forma determinista la ruta al binario de Cargo
pub fn find_cargo_executable() -> PathBuf {
    let candidate_files = [
        "/home/caja/.cargo/bin/cargo",
        "/root/.cargo/bin/cargo",
        "/usr/local/cargo/bin/cargo",
        "/usr/local/bin/cargo",
        "/usr/bin/cargo",
    ];

    for file in &candidate_files {
        let p = Path::new(file);
        if p.exists() {
            return p.to_path_buf();
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home).join(".cargo/bin/cargo");
        if p.exists() {
            return p;
        }
    }

    PathBuf::from("cargo")
}

/// Comprueba en el repositorio GitHub oficial si existe una versión más reciente
pub fn check_for_updates() -> Result<Option<RemoteUpdateInfo>, String> {
    let local_hash = get_build_git_hash().trim();
    let local_clean = local_hash.to_lowercase();
    let augmented_path = get_augmented_path();

    // Intento 1: Consultar la API de GitHub mediante curl (rápido, trae mensaje de commit y fecha)
    let curl_cmd = if cfg!(windows) { "curl.exe" } else { "curl" };
    let api_url = "https://api.github.com/repos/jesusbohada/WOLFDESK/commits/main";

    if let Ok(output) = Command::new(curl_cmd)
        .env("PATH", &augmented_path)
        .args(["-s", "--connect-timeout", "6", "-H", "User-Agent: WolfDesk", api_url])
        .output()
    {
        if output.status.success() {
            let body = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(sha) = json["sha"].as_str() {
                    let short_sha = if sha.len() >= 7 { &sha[..7] } else { sha };
                    let message = json["commit"]["message"]
                        .as_str()
                        .unwrap_or("Actualización del sistema")
                        .lines()
                        .next()
                        .unwrap_or("Actualización del sistema")
                        .to_string();
                    let date = json["commit"]["committer"]["date"]
                        .as_str()
                        .or_else(|| json["commit"]["author"]["date"].as_str())
                        .unwrap_or("")
                        .to_string();

                    let sha_clean = sha.trim().to_lowercase();
                    let short_clean = short_sha.trim().to_lowercase();

                    let mut parent_matches = false;
                    if let Some(parents) = json["parents"].as_array() {
                        for p in parents {
                            if let Some(psha) = p["sha"].as_str() {
                                let pclean = psha.trim().to_lowercase();
                                if pclean == local_clean || pclean.starts_with(&local_clean) || local_clean.starts_with(&pclean) {
                                    if message.to_lowercase().contains("release")
                                        || message.to_lowercase().contains("binario")
                                        || message.to_lowercase().contains("wolfdesk.exe")
                                    {
                                        parent_matches = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    // Comparar de forma estricta e insensible a mayúsculas/minúsculas
                    if sha_clean == local_clean
                        || short_clean == local_clean
                        || sha_clean.starts_with(&local_clean)
                        || local_clean.starts_with(&short_clean)
                        || parent_matches
                    {
                        return Ok(None); // Ya está actualizado al 100%
                    } else {
                        return Ok(Some(RemoteUpdateInfo {
                            commit_hash: sha.to_string(),
                            short_hash: short_sha.to_string(),
                            message,
                            date,
                        }));
                    }
                }
            }
        }
    }

    // Intento 2: Usar git ls-remote directamente (sin límites de rate limit de API)
    if let Ok(output) = Command::new("git")
        .env("PATH", &augmented_path)
        .args(["ls-remote", "https://github.com/jesusbohada/WOLFDESK.git", "HEAD"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().next() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(remote_sha) = parts.first() {
                    let short_sha = if remote_sha.len() >= 7 { &remote_sha[..7] } else { remote_sha };
                    let remote_clean = remote_sha.trim().to_lowercase();
                    let short_clean = short_sha.trim().to_lowercase();

                    if remote_clean == local_clean
                        || short_clean == local_clean
                        || remote_clean.starts_with(&local_clean)
                        || local_clean.starts_with(&short_clean)
                    {
                        return Ok(None);
                    } else {
                        return Ok(Some(RemoteUpdateInfo {
                            commit_hash: remote_sha.to_string(),
                            short_hash: short_sha.to_string(),
                            message: "Nueva versión disponible en la rama principal".to_string(),
                            date: "".to_string(),
                        }));
                    }
                }
            }
        }
    }

    Err("No fue posible comunicarse con el repositorio en línea de GitHub para verificar actualizaciones.".to_string())
}

/// Localiza la ruta base del repositorio o del script de actualización
fn find_repo_path() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("/home/caja/WOLFDESK"),
        PathBuf::from("/root/WOLFDESK"),
        PathBuf::from("."),
        PathBuf::from(".."),
    ];

    for candidate in &candidates {
        if candidate.join(".git").exists() {
            return Some(candidate.clone());
        }
    }

    // Comprobar ancestros del ejecutable actual
    if let Ok(current_exe) = std::env::current_exe() {
        let mut dir = current_exe.parent();
        while let Some(parent) = dir {
            if parent.join(".git").exists() {
                return Some(parent.to_path_buf());
            }
            dir = parent.parent();
        }
    }

    None
}

/// Ejecuta el proceso de actualización inteligente de WolfDesk:
/// 1. Si se detecta un repositorio de desarrollo con Cargo disponible, compila desde el código fuente.
/// 2. En equipos de usuario final, equipos nuevos o portátiles sin Git/Cargo, descarga el binario precompilado
///    directamente desde GitHub y lo sustituye en caliente sin requerir herramientas de desarrollo.
pub fn perform_update<F>(mut report_step: F) -> Result<String, String>
where
    F: FnMut(&str),
{
    report_step("Determinando método de actualización...");

    let repo_opt = find_repo_path();
    let has_cargo = {
        let augmented_path = get_augmented_path();
        Command::new(find_cargo_executable())
            .env("PATH", &augmented_path)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    };

    if let (Some(repo_dir), true) = (repo_opt, has_cargo) {
        log::info!("Entorno de desarrollo con Git y Cargo detectado. Compilando actualización...");
        perform_source_update(&repo_dir, report_step)
    } else {
        log::info!("Equipo de usuario final sin entorno de compilación. Descargando ejecutable precompilado desde GitHub...");
        perform_binary_download_update(report_step)
    }
}

/// Descarga directamente el ejecutable precompilado más reciente desde GitHub
/// y lo instala en caliente en el sistema (ideal para equipos nuevos o portátiles sin Git/Rust)
pub fn perform_binary_download_update<F>(mut report_step: F) -> Result<String, String>
where
    F: FnMut(&str),
{
    report_step("Conectando con los servidores de GitHub para descargar la actualización...");

    let exe_name = if cfg!(windows) { "wolfdesk.exe" } else { "wolfdesk" };
    let temp_dir = std::env::temp_dir();
    let pid = std::process::id();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let temp_download = temp_dir.join(format!("wolfdesk_update_{}_{}.tmp", pid, ts));
    let _ = std::fs::remove_file(&temp_download);

    // URLs oficiales en GitHub (rama main raw, blob raw y releases)
    let urls = [
        format!("https://raw.githubusercontent.com/jesusbohada/WOLFDESK/main/{}", exe_name),
        format!("https://github.com/jesusbohada/WOLFDESK/raw/main/{}", exe_name),
        format!("https://github.com/jesusbohada/WOLFDESK/releases/latest/download/{}", exe_name),
    ];

    let mut download_ok = false;
    let augmented_path = get_augmented_path();

    for url in &urls {
        report_step("Descargando el ejecutable más reciente desde GitHub...");
        let curl_cmd = if cfg!(windows) { "curl.exe" } else { "curl" };
        let res = Command::new(curl_cmd)
            .env("PATH", &augmented_path)
            .args([
                "-L",
                "-f",
                "--connect-timeout", "15",
                "-m", "180",
                "-o",
                &temp_download.to_string_lossy(),
                url,
            ])
            .output();

        if let Ok(out) = res {
            if out.status.success() && temp_download.exists() {
                if let Ok(meta) = std::fs::metadata(&temp_download) {
                    if meta.len() > 1_000_000 {
                        download_ok = true;
                        break;
                    }
                }
            }
        }

        // Respaldo para Windows si curl falló: PowerShell WebClient nativo
        #[cfg(windows)]
        {
            let ps_script = format!(
                "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; $wc = New-Object System.Net.WebClient; $wc.DownloadFile('{}', '{}')",
                url,
                temp_download.to_string_lossy().replace('\\', "\\\\")
            );
            if let Ok(out) = Command::new("powershell")
                .args(["-NoProfile", "-Command", &ps_script])
                .output()
            {
                if out.status.success() && temp_download.exists() {
                    if let Ok(meta) = std::fs::metadata(&temp_download) {
                        if meta.len() > 1_000_000 {
                            download_ok = true;
                            break;
                        }
                    }
                }
            }
        }
    }

    if !download_ok || !temp_download.exists() {
        let _ = std::fs::remove_file(&temp_download);
        return Err("No fue posible descargar la nueva versión de WolfDesk desde GitHub. Verifique su conexión a Internet.".to_string());
    }

    // Validar cabecera binaria para garantizar que la descarga es un ejecutable válido y no HTML de error
    if let Ok(bytes) = std::fs::read(&temp_download) {
        #[cfg(windows)]
        if bytes.len() < 2 || &bytes[..2] != b"MZ" {
            let _ = std::fs::remove_file(&temp_download);
            return Err("El archivo descargado no es un ejecutable de Windows válido.".to_string());
        }
        #[cfg(target_os = "linux")]
        if bytes.len() < 4 || &bytes[..4] != b"\x7fELF" {
            let _ = std::fs::remove_file(&temp_download);
            return Err("El archivo descargado no es un ejecutable de Linux válido.".to_string());
        }
    }

    report_step("Instalando nueva versión de WolfDesk en este equipo...");

    #[cfg(windows)]
    {
        // 1. Reemplazar el ejecutable actualmente activo mediante renombrado seguro
        if let Ok(cur_exe) = std::env::current_exe() {
            let old_cur = cur_exe.with_file_name(format!("{}.old_cur_{}_{}", exe_name, pid, ts));
            let _ = std::fs::remove_file(&old_cur);
            let _ = std::fs::rename(&cur_exe, &old_cur);
            if let Err(e) = std::fs::copy(&temp_download, &cur_exe) {
                let _ = std::fs::rename(&old_cur, &cur_exe);
                let _ = std::fs::remove_file(&temp_download);
                return Err(format!("Error al escribir el nuevo ejecutable en {:?}: {}", cur_exe, e));
            }
        }

        // 2. Si está en ProgramFiles
        if let Ok(prog_files) = std::env::var("ProgramFiles") {
            let pf_dest = PathBuf::from(prog_files).join("WolfDesk").join(exe_name);
            if pf_dest.exists() {
                let old_pf = pf_dest.with_file_name(format!("{}.old_pf_{}_{}", exe_name, pid, ts));
                let _ = std::fs::remove_file(&old_pf);
                let _ = std::fs::rename(&pf_dest, &old_pf);
                let _ = std::fs::copy(&temp_download, &pf_dest);
            }
        }

        // 3. Si está en LOCALAPPDATA
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            let local_dest = PathBuf::from(local_appdata).join("Programs/WolfDesk").join(exe_name);
            if local_dest.exists() {
                let old_local = local_dest.with_file_name(format!("{}.old_appdata_{}_{}", exe_name, pid, ts));
                let _ = std::fs::remove_file(&old_local);
                let _ = std::fs::rename(&local_dest, &old_local);
                let _ = std::fs::copy(&temp_download, &local_dest);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(cur_exe) = std::env::current_exe() {
            let _ = std::fs::remove_file(&cur_exe);
            let _ = std::fs::copy(&temp_download, &cur_exe);
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&cur_exe, std::fs::Permissions::from_mode(0o755));
        }
        if Path::new("/opt/wolfdesk/wolfdesk").exists() {
            let opt_dest = Path::new("/opt/wolfdesk/wolfdesk");
            let _ = std::fs::remove_file(opt_dest);
            let _ = std::fs::copy(&temp_download, opt_dest);
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(opt_dest, std::fs::Permissions::from_mode(0o755));
        }
    }

    let _ = std::fs::remove_file(&temp_download);

    let final_msg = "¡WolfDesk se ha actualizado correctamente a la última versión! Presione 'Reiniciar WolfDesk Ahora' para disfrutar de las mejoras.".to_string();
    report_step(&final_msg);
    Ok(final_msg)
}

/// Ejecuta el proceso de actualización compilando desde el código fuente (entorno de desarrollo)
pub fn perform_source_update<F>(repo_dir: &Path, mut report_step: F) -> Result<String, String>
where
    F: FnMut(&str),
{
    let augmented_path = get_augmented_path();

    // 1. Si existe scripts/update.sh en Linux, podemos invocarlo pasando el PATH adecuado
    #[cfg(target_os = "linux")]
    {
        let update_script = repo_dir.join("scripts/update.sh");
        if update_script.exists() {
            report_step("Descargando cambios y compilando con scripts/update.sh...");
            let res = Command::new("bash")
                .arg(&update_script)
                .env("PATH", &augmented_path)
                .env("WOLFDESK_IN_APP_UPDATE", "1")
                .output();
            match res {
                Ok(out) if out.status.success() => {
                    let commit_hash = Command::new("git")
                        .current_dir(&repo_dir)
                        .env("PATH", &augmented_path)
                        .args(["rev-parse", "--short", "HEAD"])
                        .output()
                        .ok()
                        .and_then(|o| String::from_utf8(o.stdout).ok())
                        .unwrap_or_else(|| "nuevo".to_string());

                    let final_msg = format!("WolfDesk actualizado exitosamente al commit [{}]. Reinicie la aplicación para aplicar.", commit_hash.trim());
                    report_step(&final_msg);
                    return Ok(final_msg);
                }
                Ok(out) => {
                    let err = String::from_utf8_lossy(&out.stderr);
                    log::warn!("Aviso en scripts/update.sh: {}", err);
                }
                Err(e) => {
                    log::warn!("No se pudo invocar scripts/update.sh: {}", e);
                }
            }
        }
    }

    report_step("Descargando últimos cambios desde GitHub...");
    let _ = Command::new("git")
        .current_dir(&repo_dir)
        .env("PATH", &augmented_path)
        .args(["fetch", "origin", "main"])
        .output();

    let _ = Command::new("git")
        .current_dir(&repo_dir)
        .env("PATH", &augmented_path)
        .args(["checkout", "-f", "main"])
        .output();

    let reset_res = Command::new("git")
        .current_dir(&repo_dir)
        .env("PATH", &augmented_path)
        .args(["reset", "--hard", "origin/main"])
        .output();
    if let Err(e) = reset_res {
        return Err(format!("Error al sincronizar con origin/main: {}", e));
    }

    // Forzar a Cargo a no cachear build.rs y recompilar con el nuevo hash
    let build_rs = repo_dir.join("crates/client-core/build.rs");
    if build_rs.exists() {
        let _ = std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&build_rs);
    }

    let git_hash = Command::new("git")
        .current_dir(&repo_dir)
        .env("PATH", &augmented_path)
        .args(["rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    #[cfg(windows)]
    {
        let target_exe = repo_dir.join("target/release/wolfdesk.exe");
        if target_exe.exists() {
            let pid = std::process::id();
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let old_exe = repo_dir.join(format!("target/release/wolfdesk.exe.old_{}_{}", pid, ts));
            let _ = std::fs::remove_file(&old_exe);
            let _ = std::fs::rename(&target_exe, &old_exe);
        }
    }

    report_step("Compilando nueva versión optimizada con Cargo (Release)...");
    let cargo_bin = find_cargo_executable();
    let cargo_res = Command::new(&cargo_bin)
        .current_dir(&repo_dir)
        .env("PATH", &augmented_path)
        .env("WOLFDESK_BUILD_GIT_HASH", &git_hash)
        .args(["build", "--release", "--bin", "wolfdesk"])
        .output();

    match cargo_res {
        Ok(out) if out.status.success() => {
            log::info!("Compilación de actualización exitosa.");
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(format!("Fallo en la compilación de Cargo: {}", stderr));
        }
        Err(e) => {
            return Err(format!("Error al invocar cargo ({:?}): {}", cargo_bin, e));
        }
    }

    let exe_name = if cfg!(windows) { "wolfdesk.exe" } else { "wolfdesk" };
    let new_binary = repo_dir.join("target/release").join(exe_name);
    if !new_binary.exists() {
        return Err("No se encontró el binario compilado en target/release/wolfdesk".to_string());
    }

    report_step("Instalando nuevo ejecutable en el sistema...");

    #[cfg(target_os = "linux")]
    {
        let root_bin = repo_dir.join(exe_name);
        let _ = std::fs::remove_file(&root_bin);
        let _ = std::fs::copy(&new_binary, &root_bin);

        if Path::new("/opt/wolfdesk").exists() {
            let dest = Path::new("/opt/wolfdesk/wolfdesk");
            let _ = std::fs::remove_file(dest);
            let _ = std::fs::copy(&new_binary, dest);
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o755));
        }
        if Path::new("/etc/systemd/system/wolfdesk.service").exists() {
            let _ = Command::new("systemctl").arg("daemon-reload").output();
        }
    }

    #[cfg(windows)]
    {
        let pid = std::process::id();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // 1. Actualizar wolfdesk.exe en la raíz del repositorio
        let root_exe = repo_dir.join(exe_name);
        if root_exe.exists() {
            let old_root = repo_dir.join(format!("{}.old_root_{}_{}", exe_name, pid, ts));
            let _ = std::fs::remove_file(&old_root);
            let _ = std::fs::rename(&root_exe, &old_root);
            let _ = std::fs::copy(&new_binary, &root_exe);
        } else {
            let _ = std::fs::copy(&new_binary, &root_exe);
        }

        // 2. Actualizar el ejecutable actualmente en ejecución
        if let Ok(cur_exe) = std::env::current_exe() {
            if cur_exe.exists() && cur_exe != new_binary && cur_exe != root_exe {
                let old_cur = cur_exe.with_file_name(format!("{}.old_cur_{}_{}", exe_name, pid, ts));
                let _ = std::fs::remove_file(&old_cur);
                let _ = std::fs::rename(&cur_exe, &old_cur);
                let _ = std::fs::copy(&new_binary, &cur_exe);
            }
        }

        // 3. Actualizar instalación formal en ProgramFiles
        if let Ok(prog_files) = std::env::var("ProgramFiles") {
            let dest = PathBuf::from(prog_files).join("WolfDesk").join(exe_name);
            if dest.exists() && dest != new_binary {
                let old_dest = dest.with_file_name(format!("{}.old_pf_{}_{}", exe_name, pid, ts));
                let _ = std::fs::remove_file(&old_dest);
                let _ = std::fs::rename(&dest, &old_dest);
                let _ = std::fs::copy(&new_binary, &dest);
            }
        }

        // 4. Actualizar instalación formal en LOCALAPPDATA
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            let dest = PathBuf::from(local_appdata).join("Programs/WolfDesk").join(exe_name);
            if dest.exists() && dest != new_binary {
                let old_dest = dest.with_file_name(format!("{}.old_appdata_{}_{}", exe_name, pid, ts));
                let _ = std::fs::remove_file(&old_dest);
                let _ = std::fs::rename(&dest, &old_dest);
                let _ = std::fs::copy(&new_binary, &dest);
            }
        }
    }

    let commit_hash = Command::new("git")
        .current_dir(&repo_dir)
        .env("PATH", &augmented_path)
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "nuevo".to_string());

    let final_msg = format!("WolfDesk actualizado exitosamente al commit [{}]. Reinicie la aplicación para disfrutar de las mejoras.", commit_hash.trim());
    report_step(&final_msg);
    Ok(final_msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_version_and_hash() {
        assert!(!get_local_version().is_empty());
        assert!(!get_build_git_hash().is_empty());
    }

    #[test]
    fn test_augmented_path() {
        let path = get_augmented_path();
        assert!(!path.is_empty());
    }

    #[test]
    fn test_update_status_variants() {
        let status = UpdateStatus::NotChecked;
        assert_eq!(status, UpdateStatus::NotChecked);

        let status2 = UpdateStatus::UpToDate {
            commit: "10eb531".to_string(),
            checked_time: "12:00".to_string(),
        };
        assert!(matches!(status2, UpdateStatus::UpToDate { .. }));
    }

    #[test]
    fn test_check_for_updates_live() {
        let res = check_for_updates();
        println!("check_for_updates res = {:?}", res);
        assert!(res.is_ok());
        let update_opt = res.unwrap();
        assert!(update_opt.is_none(), "Debe detectar que el sistema está completamente actualizado");
    }
}
