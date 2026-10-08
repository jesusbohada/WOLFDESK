use proto::SessionPermissions;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

const CONFIG_FILE: &str = "remote_desktop_config.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    /// URL del servidor de señalización (local o público en la nube)
    pub server_url: String,
    /// Servidores STUN públicos para travesía NAT (Internet)
    pub stun_servers: Vec<String>,
    /// Hash de contraseña para acceso desatendido (sin interacción humana)
    pub unattended_password_hash: Option<String>,
    /// Permisos predeterminados que se conceden
    pub default_permissions: SessionPermissions,
    /// Historial de puestos remotos recientes
    pub recent_ids: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_url: "ws://127.0.0.1:9050/ws".to_string(),
            stun_servers: vec![
                "stun.l.google.com:19302".to_string(),
                "stun1.l.google.com:19302".to_string(),
                "stun.cloudflare.com:3478".to_string(),
            ],
            unattended_password_hash: None,
            default_permissions: SessionPermissions::default(),
            recent_ids: Vec::new(),
        }
    }
}

impl AppConfig {
    /// Carga la configuración desde el disco o crea una por defecto
    pub fn load() -> Self {
        if Path::new(CONFIG_FILE).exists() {
            if let Ok(content) = fs::read_to_string(CONFIG_FILE) {
                if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                    return config;
                }
            }
        }
        let default_config = Self::default();
        let _ = default_config.save();
        default_config
    }

    /// Guarda la configuración actual en el disco
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(CONFIG_FILE, json)?;
        Ok(())
    }

    /// Configura una nueva contraseña de acceso desatendido
    pub fn set_unattended_password(&mut self, password: &str) {
        if password.trim().is_empty() {
            self.unattended_password_hash = None;
        } else {
            self.unattended_password_hash = Some(Self::hash_password(password));
        }
        let _ = self.save();
    }

    /// Comprueba si una contraseña coincide con la configurada
    pub fn verify_password(&self, password: &str) -> bool {
        match &self.unattended_password_hash {
            Some(expected_hash) => &Self::hash_password(password) == expected_hash,
            None => false,
        }
    }

    /// Añade un ID al historial de conexiones recientes
    pub fn add_recent_id(&mut self, id: &str) {
        let clean = id.trim().to_string();
        if !clean.is_empty() && !self.recent_ids.contains(&clean) {
            self.recent_ids.insert(0, clean);
            if self.recent_ids.len() > 10 {
                self.recent_ids.pop();
            }
            let _ = self.save();
        }
    }

    fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"RemoteDesktopSalt_");
        hasher.update(password.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}
