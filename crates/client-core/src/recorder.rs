use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

const RECORDINGS_DIR: &str = "WolfDesk_Recordings";

pub struct SessionRecorder {
    pub is_recording: bool,
    current_dir: Option<PathBuf>,
    frame_index: u64,
    start_time: Option<Instant>,
}

impl SessionRecorder {
    pub fn new() -> Self {
        let _ = fs::create_dir_all(RECORDINGS_DIR);
        Self {
            is_recording: false,
            current_dir: None,
            frame_index: 0,
            start_time: None,
        }
    }

    /// Inicia la grabación de la sesión actual
    pub fn start(&mut self, session_title: &str) -> PathBuf {
        let timestamp = chrono_timestamp();
        let folder_name = format!("grabacion_{}_{}", session_title.replace(' ', "_"), timestamp);
        let folder_path = Path::new(RECORDINGS_DIR).join(folder_name);
        let _ = fs::create_dir_all(&folder_path);

        self.current_dir = Some(folder_path.clone());
        self.frame_index = 0;
        self.start_time = Some(Instant::now());
        self.is_recording = true;

        log::info!("Grabación iniciada en: {:?}", folder_path);
        folder_path
    }

    /// Guarda un fotograma individual recibido en el disco
    pub fn record_frame(&mut self, jpeg_bytes: &[u8]) {
        if !self.is_recording {
            return;
        }

        if let Some(ref dir) = self.current_dir {
            let file_name = format!("frame_{:06}.jpg", self.frame_index);
            let frame_path = dir.join(file_name);
            if let Ok(mut f) = File::create(frame_path) {
                let _ = f.write_all(jpeg_bytes);
                self.frame_index += 1;
            }
        }
    }

    /// Detiene la grabación y genera un informe de auditoría
    pub fn stop(&mut self) -> Option<(u64, u64, PathBuf)> {
        if !self.is_recording {
            return None;
        }

        self.is_recording = false;
        let elapsed_secs = self.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0);
        let total_frames = self.frame_index;

        if let Some(ref dir) = self.current_dir {
            let info_path = dir.join("info_sesion.txt");
            if let Ok(mut f) = File::create(&info_path) {
                let _ = writeln!(f, "=== INFORME DE SESIÓN WOLFDESK ===");
                let _ = writeln!(f, "Total de fotogramas guardados: {}", total_frames);
                let _ = writeln!(f, "Duración total: {} segundos", elapsed_secs);
                let _ = writeln!(f, "FPS promedio: {:.1}", if elapsed_secs > 0 { total_frames as f64 / elapsed_secs as f64 } else { 0.0 });
            }
            let res = (total_frames, elapsed_secs, dir.clone());
            self.current_dir = None;
            return Some(res);
        }
        None
    }
}

fn chrono_timestamp() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    now.to_string()
}
