use proto::{Identity, IdentityStorage};
use std::fs;
use std::path::PathBuf;

/// Obtiene las rutas candidatas donde se almacena la identidad persistente de WolfDesk
pub fn get_identity_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(windows)]
    {
        // 1. Ubicación estándar del sistema para software instalado (ProgramData)
        if let Ok(progdata) = std::env::var("ProgramData") {
            paths.push(PathBuf::from(progdata).join("WolfDesk").join("identity.json"));
        }
        // 2. Ubicación por usuario (AppData/Roaming)
        if let Ok(appdata) = std::env::var("APPDATA") {
            paths.push(PathBuf::from(appdata).join("WolfDesk").join("identity.json"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        paths.push(PathBuf::from("/etc/wolfdesk/identity.json"));
        if let Ok(home) = std::env::var("HOME") {
            paths.push(PathBuf::from(home).join(".config").join("wolfdesk").join("identity.json"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/Library/Application Support/WolfDesk/identity.json"));
        if let Ok(home) = std::env::var("HOME") {
            paths.push(PathBuf::from(home).join("Library").join("Application Support").join("WolfDesk").join("identity.json"));
        }
    }

    // 3. Ubicación local portátil (carpeta actual donde reside wolfdesk.exe)
    paths.push(PathBuf::from("identity.json"));

    paths
}

/// Carga la identidad persistente existente o genera una nueva fija en disco.
/// El ID derivado será permanente y nunca cambiará a menos que se desinstale la aplicación.
pub fn load_or_create_identity() -> Identity {
    let candidate_paths = get_identity_paths();

    // 1. Intentar cargar desde cualquiera de las rutas existentes
    for path in &candidate_paths {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(storage) = serde_json::from_str::<IdentityStorage>(&content) {
                    if let Some(identity) = Identity::from_storage(&storage) {
                        log::info!("Identidad fija de WolfDesk cargada desde [{:?}]: ID={}", path, identity.numeric_id);
                        return identity;
                    }
                }
            }
        }
    }

    // 2. Si no existe ninguna identidad previa, generamos una nueva única
    let new_identity = Identity::generate();
    let storage = new_identity.to_storage();

    if let Ok(json) = serde_json::to_string_pretty(&storage) {
        // Guardamos en la primera ruta escribible disponible
        for path in &candidate_paths {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if fs::write(path, &json).is_ok() {
                log::info!("Nueva identidad fija de WolfDesk guardada en [{:?}]: ID={}", path, new_identity.numeric_id);
                break;
            }
        }
    }

    new_identity
}

/// Elimina la identidad persistente al desinstalar la aplicación
pub fn remove_identity_on_uninstall() {
    for path in get_identity_paths() {
        if path.exists() {
            let _ = fs::remove_file(&path);
            if let Some(parent) = path.parent() {
                // Intentar limpiar la carpeta si está vacía
                let _ = fs::remove_dir(parent);
            }
        }
    }
}
