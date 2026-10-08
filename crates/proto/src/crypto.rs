use ed25519_dalek::{SigningKey, VerifyingKey, Signer, Verifier, Signature};
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};

/// Identidad criptográfica local de un cliente
pub struct Identity {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
    pub numeric_id: String,
}

impl Identity {
    /// Genera una nueva identidad criptográfica segura basada en Ed25519
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let numeric_id = Self::derive_numeric_id(&verifying_key);

        Self {
            signing_key,
            verifying_key,
            numeric_id,
        }
    }

    /// Deriva un ID amigable de 9 dígitos (estilo AnyDesk: XXX XXX XXX)
    /// a partir del hash SHA-256 de la clave pública del host.
    pub fn derive_numeric_id(verifying_key: &VerifyingKey) -> String {
        let mut hasher = Sha256::new();
        hasher.update(verifying_key.as_bytes());
        let hash = hasher.finalize();

        // Convertimos los primeros 4 bytes del hash en un número de 9 dígitos (100_000_000 a 999_999_999)
        let num = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]);
        let id_val = 100_000_000 + (num % 900_000_000);
        let s = id_val.to_string();

        format!("{} {} {}", &s[0..3], &s[3..6], &s[6..9])
    }

    /// Obtiene la clave pública en formato hexadecimal
    pub fn public_key_hex(&self) -> String {
        hex_encode(self.verifying_key.as_bytes())
    }

    /// Obtiene la clave privada en formato hexadecimal para guardado seguro
    pub fn private_key_hex(&self) -> String {
        hex_encode(&self.signing_key.to_bytes())
    }

    /// Reconstruye una identidad a partir de 32 bytes de clave privada
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(bytes);
        let verifying_key = signing_key.verifying_key();
        let numeric_id = Self::derive_numeric_id(&verifying_key);
        Self {
            signing_key,
            verifying_key,
            numeric_id,
        }
    }

    /// Reconstruye una identidad a partir de una cadena hexadecimal de la clave privada
    pub fn from_private_hex(hex_str: &str) -> Option<Self> {
        let bytes = hex_decode(hex_str)?;
        if bytes.len() != 32 {
            return None;
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Some(Self::from_bytes(&arr))
    }

    /// Convierte la identidad a estructura serializable para almacenamiento persistente
    pub fn to_storage(&self) -> IdentityStorage {
        IdentityStorage {
            numeric_id: self.numeric_id.clone(),
            private_key_hex: self.private_key_hex(),
            public_key_hex: self.public_key_hex(),
        }
    }

    /// Carga la identidad desde la estructura persistente almacenada
    pub fn from_storage(storage: &IdentityStorage) -> Option<Self> {
        Self::from_private_hex(&storage.private_key_hex)
    }

    /// Firma un mensaje o token de autenticación
    pub fn sign_message(&self, message: &[u8]) -> String {
        let signature = self.signing_key.sign(message);
        hex_encode(&signature.to_bytes())
    }
}

/// Estructura serializable en JSON para almacenar la identidad permanente en disco
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct IdentityStorage {
    pub numeric_id: String,
    pub private_key_hex: String,
    pub public_key_hex: String,
}

/// Verifica una firma Ed25519 con una clave pública dada
pub fn verify_signature(public_key_hex: &str, message: &[u8], signature_hex: &str) -> bool {
    let pk_bytes = match hex_decode(public_key_hex) {
        Some(b) if b.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&b);
            arr
        }
        _ => return false,
    };

    let sig_bytes = match hex_decode(signature_hex) {
        Some(b) if b.len() == 64 => {
            let mut arr = [0u8; 64];
            arr.copy_from_slice(&b);
            arr
        }
        _ => return false,
    };

    let verifying_key = match VerifyingKey::from_bytes(&pk_bytes) {
        Ok(k) => k,
        Err(_) => return false,
    };

    let signature = Signature::from_bytes(&sig_bytes);
    verifying_key.verify(message, &signature).is_ok()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_and_signature() {
        let id = Identity::generate();
        assert_eq!(id.numeric_id.len(), 11); // 9 dígitos + 2 espacios

        let msg = b"RemoteDesktopAuthChallenge_12345";
        let sig = id.sign_message(msg);
        assert!(verify_signature(&id.public_key_hex(), msg, &sig));
        assert!(!verify_signature(&id.public_key_hex(), b"DifferentMessage", &sig));
    }
}
