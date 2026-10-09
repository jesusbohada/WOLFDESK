use serde::{Deserialize, Serialize};

/// Permisos específicos que el host otorga a una sesión remota
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionPermissions {
    pub allow_mouse: bool,
    pub allow_keyboard: bool,
    pub allow_clipboard: bool,
    pub allow_file_transfer: bool,
}

impl Default for SessionPermissions {
    fn default() -> Self {
        Self {
            allow_mouse: true,
            allow_keyboard: true,
            allow_clipboard: true,
            allow_file_transfer: true,
        }
    }
}

/// Mensajes de señalización y control de sesión
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum SignalMessage {
    /// Registro de cliente con ID y clave pública
    Register {
        client_id: String,
        public_key: String,
    },
    /// Confirmación de registro exitoso
    RegisterSuccess {
        assigned_id: String,
    },
    /// Solicitud de conexión
    ConnectRequest {
        target_id: String,
        password_hash: Option<String>,
        auth_signature: Option<String>,
    },
    /// Notificación al Host para mostrar el diálogo de "¿Aceptar o Rechazar?"
    IncomingPrompt {
        from_id: String,
    },
    /// El Host acepta con permisos específicos
    ConnectAccept {
        from_id: String,
        permissions: SessionPermissions,
    },
    /// El Host rechaza la conexión
    ConnectReject {
        from_id: String,
        reason: String,
    },
    /// Cierre voluntario de sesión
    Disconnect {
        target_id: String,
        reason: String,
    },
    /// Mensaje de Chat integrado entre usuarios
    Chat {
        target_id: String,
        text: String,
    },
    /// Inicio de transferencia de archivo
    FileTransferStart {
        target_id: String,
        file_name: String,
        file_size: u64,
        total_chunks: u32,
    },
    /// Bloque de datos de archivo
    FileTransferChunk {
        target_id: String,
        file_name: String,
        chunk_index: u32,
        total_chunks: u32,
        data_base64: String,
    },
    /// Fin de transferencia de archivo
    FileTransferComplete {
        target_id: String,
        file_name: String,
    },
    /// Solicitud de listado de archivos en una ruta remota
    FileListRequest {
        target_id: String,
        path: String,
    },
    /// Respuesta con el listado de archivos del equipo remoto
    FileListResponse {
        target_id: String,
        path: String,
        entries: Vec<FileEntry>,
    },
    /// Solicitud para descargar un archivo desde el equipo remoto
    FileDownloadRequest {
        target_id: String,
        remote_file_path: String,
    },
    /// Streaming de fotograma comprimido (Base64 JPEG)
    VideoFrame {
        target_id: String,
        width: u32,
        height: u32,
        jpeg_base64: String,
    },
    /// Eventos de control remoto
    Control {
        target_id: String,
        event: ControlEvent,
    },
    /// Oferta WebRTC
    Offer {
        target_id: String,
        sdp: String,
    },
    /// Respuesta WebRTC
    Answer {
        target_id: String,
        sdp: String,
    },
    /// Candidatos ICE
    IceCandidate {
        target_id: String,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: Option<u16>,
    },
    /// Mensaje de error general
    Error {
        message: String,
    },
}

/// Eventos de control de periféricos y ajustes de calidad en vivo
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "action", content = "data")]
pub enum ControlEvent {
    MouseMove {
        x: f32,
        y: f32,
    },
    MouseButton {
        button: MouseButtonType,
        down: bool,
        #[serde(default)]
        x: Option<f32>,
        #[serde(default)]
        y: Option<f32>,
    },
    MouseWheel {
        delta_x: i32,
        delta_y: i32,
    },
    Keyboard {
        vk_code: u16,
        down: bool,
    },
    KeyboardUnicode {
        text: String,
    },
    SelectDisplay {
        display_id: u8,
    },
    ClipboardSync {
        text: String,
    },
    /// Cambio dinámico de calidad de imagen (ej: 40 para velocidad, 90 para HD)
    SetQuality {
        quality: u8,
    },
    /// Límite dinámico de tasa de fotogramas (0 = sin límite)
    SetFpsLimit {
        fps: u32,
    },
    Ping {
        timestamp: u64,
    },
    Pong {
        timestamp: u64,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButtonType {
    Left,
    Right,
    Middle,
}

/// Representa una entrada de archivo o carpeta en el explorador
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size_bytes: u64,
}
