use base64::Engine;
use proto::SignalMessage;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

const CHUNK_SIZE: usize = 64 * 1024; // 64 KB por bloque
const DOWNLOADS_DIR: &str = "WolfDesk_Downloads";

pub struct IncomingFile {
    pub file_name: String,
    pub file_size: u64,
    pub total_chunks: u32,
    pub chunks: HashMap<u32, Vec<u8>>,
}

pub struct FileTransferManager {
    incoming: HashMap<String, IncomingFile>,
}

impl FileTransferManager {
    pub fn new() -> Self {
        let _ = fs::create_dir_all(DOWNLOADS_DIR);
        Self {
            incoming: HashMap::new(),
        }
    }

    /// Inicia el envío de un archivo local hacia el puesto remoto por bloques
    pub async fn send_file(
        file_path: &Path,
        target_id: &str,
        tx: &mpsc::UnboundedSender<SignalMessage>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut file = File::open(file_path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len();
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("archivo_recibido.dat")
            .to_string();

        let total_chunks = ((file_size as f64) / (CHUNK_SIZE as f64)).ceil() as u32;

        // 1. Enviar aviso de inicio
        let _ = tx.send(SignalMessage::FileTransferStart {
            target_id: target_id.to_string(),
            file_name: file_name.clone(),
            file_size,
            total_chunks,
        });

        // 2. Transmitir bloques de 64 KB codificados en Base64
        let mut buffer = vec![0u8; CHUNK_SIZE];
        let mut chunk_index = 0;

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let chunk_data = &buffer[..bytes_read];
            let data_base64 = base64::engine::general_purpose::STANDARD.encode(chunk_data);

            let _ = tx.send(SignalMessage::FileTransferChunk {
                target_id: target_id.to_string(),
                file_name: file_name.clone(),
                chunk_index,
                total_chunks,
                data_base64,
            });

            chunk_index += 1;
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }

        // 3. Notificar fin de transmisión
        let _ = tx.send(SignalMessage::FileTransferComplete {
            target_id: target_id.to_string(),
            file_name: file_name.clone(),
        });

        Ok(format!("Archivo '{}' enviado ({:.2} MB)", file_name, file_size as f64 / 1_048_576.0))
    }

    /// Procesa el inicio de una transferencia entrante
    pub fn handle_start(&mut self, file_name: &str, file_size: u64, total_chunks: u32) {
        self.incoming.insert(
            file_name.to_string(),
            IncomingFile {
                file_name: file_name.to_string(),
                file_size,
                total_chunks,
                chunks: HashMap::new(),
            },
        );
    }

    /// Almacena un bloque recibido
    pub fn handle_chunk(&mut self, file_name: &str, chunk_index: u32, data_base64: &str) {
        if let Some(incoming) = self.incoming.get_mut(file_name) {
            if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(data_base64) {
                incoming.chunks.insert(chunk_index, bytes);
            }
        }
    }

    /// Reensambla el archivo completo y lo guarda en WolfDesk_Downloads
    pub fn handle_complete(&mut self, file_name: &str) -> Option<PathBuf> {
        if let Some(incoming) = self.incoming.remove(file_name) {
            let output_path = Path::new(DOWNLOADS_DIR).join(&incoming.file_name);
            if let Ok(mut out_file) = File::create(&output_path) {
                for i in 0..incoming.total_chunks {
                    if let Some(data) = incoming.chunks.get(&i) {
                        let _ = out_file.write_all(data);
                    }
                }
                return Some(output_path);
            }
        }
        None
    }

    /// Obtiene la ruta del directorio Home del usuario del sistema operativo (Windows / Linux)
    pub fn get_user_home_dir() -> String {
        #[cfg(windows)]
        {
            if let Ok(prof) = std::env::var("USERPROFILE") {
                if Path::new(&prof).exists() {
                    return prof;
                }
            }
            if let (Ok(drive), Ok(path)) = (std::env::var("HOMEDRIVE"), std::env::var("HOMEPATH")) {
                let combined = format!("{}{}", drive, path);
                if Path::new(&combined).exists() {
                    return combined;
                }
            }
        }
        #[cfg(not(windows))]
        {
            if let Ok(home) = std::env::var("HOME") {
                if Path::new(&home).exists() {
                    return home;
                }
            }
        }
        ".".to_string()
    }

    /// Lista carpetas y archivos en una ruta de forma ordenada (carpetas primero)
    pub fn list_directory(target_path: &str) -> (String, Vec<proto::FileEntry>) {
        let trimmed = target_path.trim();
        let p = if trimmed.is_empty() || trimmed == "~" || trimmed == "HOME" {
            PathBuf::from(Self::get_user_home_dir())
        } else {
            PathBuf::from(trimmed)
        };

        let resolved_path = match p.canonicalize() {
            Ok(cp) => cp.to_string_lossy().to_string().trim_start_matches(r"\\?\").to_string(),
            Err(_) => p.to_string_lossy().to_string(),
        };

        let mut entries = Vec::new();
        if let Ok(read_dir) = fs::read_dir(&p) {
            for entry in read_dir.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                entries.push(proto::FileEntry {
                    name,
                    is_dir,
                    size_bytes,
                });
            }
        }

        // Ordenar: primero carpetas alfabéticamente, luego archivos alfabéticamente
        entries.sort_by(|a, b| {
            b.is_dir.cmp(&a.is_dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        (resolved_path, entries)
    }
}
