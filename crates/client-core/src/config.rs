use proto::SessionPermissions;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

const CONFIG_FILE: &str = "remote_desktop_config.json";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Contact {
    pub id: String,
    pub alias: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub created_at: String,
}

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
    /// Libreta de direcciones / contactos guardados
    #[serde(default)]
    pub contacts: Vec<Contact>,
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
            contacts: Vec::new(),
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

    /// Guarda o actualiza un contacto en la libreta de direcciones
    pub fn save_contact(&mut self, id: &str, alias: &str, notes: &str) {
        let clean_id = id.trim().to_string();
        let clean_alias = if alias.trim().is_empty() {
            format!("Equipo {}", clean_id)
        } else {
            alias.trim().to_string()
        };
        let clean_notes = notes.trim().to_string();

        if clean_id.is_empty() {
            return;
        }

        if let Some(existing) = self.contacts.iter_mut().find(|c| c.id == clean_id) {
            existing.alias = clean_alias;
            existing.notes = clean_notes;
        } else {
            let timestamp = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                Ok(d) => format!("{}", d.as_secs()),
                Err(_) => "".to_string(),
            };
            self.contacts.push(Contact {
                id: clean_id,
                alias: clean_alias,
                notes: clean_notes,
                created_at: timestamp,
            });
        }
        let _ = self.save();
    }

    /// Elimina un contacto de la libreta
    pub fn remove_contact(&mut self, id: &str) {
        self.contacts.retain(|c| c.id != id);
        let _ = self.save();
    }

    fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"RemoteDesktopSalt_");
        hasher.update(password.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contacts_crud() {
        let mut cfg = AppConfig::default();
        assert!(cfg.contacts.is_empty());

        cfg.save_contact("123456789", "Oficina", "Planta 2");
        assert_eq!(cfg.contacts.len(), 1);
        assert_eq!(cfg.contacts[0].id, "123456789");
        assert_eq!(cfg.contacts[0].alias, "Oficina");
        assert_eq!(cfg.contacts[0].notes, "Planta 2");

        // Actualizar alias y notas del mismo contacto
        cfg.save_contact("123456789", "Oficina Central", "Piso 5");
        assert_eq!(cfg.contacts.len(), 1);
        assert_eq!(cfg.contacts[0].alias, "Oficina Central");
        assert_eq!(cfg.contacts[0].notes, "Piso 5");

        // Añadir otro contacto
        cfg.save_contact("987654321", "PC Casa", "");
        assert_eq!(cfg.contacts.len(), 2);

        // Eliminar primer contacto
        cfg.remove_contact("123456789");
        assert_eq!(cfg.contacts.len(), 1);
        assert_eq!(cfg.contacts[0].id, "987654321");
    }
}
