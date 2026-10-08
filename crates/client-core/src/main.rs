#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use client_core::{
    dashboard::{DashboardApp, NetToUi, UiToNet},
    dispatch_event_with_permissions, load_or_create_identity, start_viewer_window,
    AppConfig, FileTransferManager, ScreenCapturer, SignalingClient,
};
use eframe::egui::{self, Vec2, ViewportBuilder};
use proto::{ControlEvent, SessionPermissions, SignalMessage};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;

fn load_app_icon() -> Option<egui::IconData> {
    let bytes = std::fs::read("assets/wolfdesk.png")
        .or_else(|_| std::fs::read("crates/client-core/assets/wolfdesk.png"))
        .ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}

fn main() -> Result<(), eframe::Error> {
    // 0. Comprobar parámetros de línea de comandos para instalación o desinstalación silenciosa
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--install") {
        match client_core::install_to_system() {
            Ok(msg) => println!("{}", msg),
            Err(e) => eprintln!("Error de instalación: {}", e),
        }
        return Ok(());
    }
    if args.iter().any(|a| a == "--uninstall") {
        match client_core::uninstall_from_system() {
            Ok(msg) => println!("{}", msg),
            Err(e) => eprintln!("Error de desinstalación: {}", e),
        }
        return Ok(());
    }

    // Instancia única: si WolfDesk ya está activo en segundo plano, solicitar mostrar la ventana y salir
    let single_instance_port = 19059;
    if let Ok(socket) = std::net::UdpSocket::bind("127.0.0.1:0") {
        let _ = socket.set_read_timeout(Some(std::time::Duration::from_millis(300)));
        let _ = socket.send_to(b"SHOW", format!("127.0.0.1:{}", single_instance_port));
        let mut buf = [0u8; 16];
        if let Ok((len, _)) = socket.recv_from(&mut buf) {
            if &buf[..len] == b"OK" {
                println!("🐺 WolfDesk ya está activo en segundo plano. Restaurando ventana al primer plano...");
                return Ok(());
            }
        }
    }

    #[cfg(windows)]
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::SetProcessDPIAware();
    }

    #[cfg(target_os = "linux")]
    {
        if std::env::var("DISPLAY").is_err() {
            std::env::set_var("DISPLAY", ":0");
        }
        if std::env::var("XAUTHORITY").is_err() {
            let candidates = ["/home/caja/.Xauthority", "/root/.Xauthority"];
            for path in &candidates {
                if std::path::Path::new(path).exists() {
                    std::env::set_var("XAUTHORITY", path);
                    break;
                }
            }
            if std::env::var("XAUTHORITY").is_err() {
                if let Ok(entries) = std::fs::read_dir("/home") {
                    for entry in entries.flatten() {
                        let p = entry.path().join(".Xauthority");
                        if p.exists() {
                            std::env::set_var("XAUTHORITY", p.to_string_lossy().to_string());
                            break;
                        }
                    }
                }
            }
        }
    }

    // Inicializar el logger integrado para la terminal GUI interna
    client_core::init_terminal_logger();

    let config = AppConfig::load();
    // Carga o generación de la Identidad Fija Permanente del equipo (Ed25519)
    let identity = load_or_create_identity();
    let my_id = identity.numeric_id.clone();
    let client = SignalingClient::new(&config.server_url, identity);

    // Canales bidireccionales entre la GUI (hilo principal) y el Motor de Red (Tokio)
    let (tx_to_net, mut rx_from_ui) = mpsc::unbounded_channel::<UiToNet>();
    let (tx_to_ui, rx_from_net) = std::sync::mpsc::channel::<NetToUi>();

    // Iniciar el icono de bandeja del sistema (segundo plano permanente junto al reloj)
    client_core::tray::start_system_tray(&my_id, tx_to_ui.clone());

    // Escuchar solicitudes de mostrar ventana desde instancias secundarias
    if let Ok(listener) = std::net::UdpSocket::bind(format!("127.0.0.1:{}", single_instance_port)) {
        let tx_ui = tx_to_ui.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 32];
            while let Ok((len, src)) = listener.recv_from(&mut buf) {
                if &buf[..len] == b"SHOW" {
                    let _ = listener.send_to(b"OK", src);
                    let _ = tx_ui.send(NetToUi::ShowWindow);
                    client_core::tray::wake_ui();
                }
            }
        });
    }

    // Canales de red hacia el servidor WebSocket
    let (outbound_tx, outbound_rx) = mpsc::unbounded_channel::<SignalMessage>();
    let (inbound_tx, mut inbound_rx) = mpsc::unbounded_channel::<SignalMessage>();

    // Canal para transmitir eventos de mouse/teclado y calidad desde la ventana del visor
    let (viewer_control_tx, mut viewer_control_rx) = mpsc::unbounded_channel::<ControlEvent>();

    // Canal síncrono para enviar fotogramas de video recibidos a la ventana del visor
    let viewer_frame_tx: Arc<Mutex<Option<std::sync::mpsc::Sender<Vec<u8>>>>> = Arc::new(Mutex::new(None));

    // Estado de la sesión remota activa
    let active_controller_id: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
    let active_target_id: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
    let active_permissions: Arc<RwLock<SessionPermissions>> = Arc::new(RwLock::new(SessionPermissions::default()));
    let shared_config: Arc<RwLock<AppConfig>> = Arc::new(RwLock::new(config.clone()));
    let active_streaming_quality: Arc<RwLock<u8>> = Arc::new(RwLock::new(88)); // 88% HD Nativa cristalina por defecto

    // Gestor de transferencia de archivos
    let file_manager: Arc<Mutex<FileTransferManager>> = Arc::new(Mutex::new(FileTransferManager::new()));

    // 1. Runtime Tokio en hilo secundario
    let net_identity_id = my_id.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let _signaling_handle = tokio::spawn({
                let client_ref = client;
                async move {
                    if let Err(e) = client_ref.start(outbound_rx, inbound_tx).await {
                        log::error!("Error en conexión de señalización WolfDesk: {}", e);
                    }
                }
            });

            // Reenvío de eventos del visor (Mouse, Teclado, Calidad) hacia el Host
            let outbound_tx_for_controls = outbound_tx.clone();
            let target_for_controls = active_target_id.clone();
            tokio::spawn(async move {
                while let Some(event) = viewer_control_rx.recv().await {
                    if let Some(ref target) = *target_for_controls.read().await {
                        let _ = outbound_tx_for_controls.send(SignalMessage::Control {
                            target_id: target.clone(),
                            event,
                        });
                    }
                }
            });

            // Bucle de captura y streaming cuando este equipo actúa como Host
            let active_controller_for_capture = active_controller_id.clone();
            let tx_for_capture = outbound_tx.clone();
            let quality_for_capture = active_streaming_quality.clone();

            tokio::spawn(async move {
                use base64::Engine;
                let mut capturer = ScreenCapturer::new();

                loop {
                    sleep(Duration::from_millis(60)).await; // ~16 FPS (fluido, baja latencia y consumo óptimo)
                    let maybe_controller = active_controller_for_capture.read().await.clone();
                    if let Some(target) = maybe_controller {
                        let current_q = *quality_for_capture.read().await;
                        match capturer.capture_frame(current_q) {
                            Some(frame) => {
                                static LOGGED_STREAM: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                                if !LOGGED_STREAM.swap(true, std::sync::atomic::Ordering::Relaxed) {
                                    log::info!("🚀 [HOST STREAMING] Transmitiendo escritorio en vivo a [{}] ({}x{})", target, frame.width, frame.height);
                                }
                                let base64_str = base64::engine::general_purpose::STANDARD.encode(&frame.jpeg_bytes);
                                let _ = tx_for_capture.send(SignalMessage::VideoFrame {
                                    target_id: target,
                                    width: frame.width,
                                    height: frame.height,
                                    jpeg_base64: base64_str,
                                });
                            }
                            None => {
                                static LOGGED_NONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                                if !LOGGED_NONE.swap(true, std::sync::atomic::Ordering::Relaxed) {
                                    log::warn!("⚠️ [HOST STREAMING] capturer.capture_frame() retornó None. Compruebe permisos de X11 en la pestaña Terminal.");
                                }
                            }
                        }
                    }
                }
            });

            // Procesar peticiones emitidas desde la GUI de WolfDesk
            let outbound_for_ui = outbound_tx.clone();
            let target_for_ui = active_target_id.clone();
            let active_ctrl_for_ui = active_controller_id.clone();
            let active_perm_for_ui = active_permissions.clone();
            let shared_cfg_for_ui = shared_config.clone();
            let tx_ui_notify_cmd = tx_to_ui.clone();

            tokio::spawn(async move {
                while let Some(cmd) = rx_from_ui.recv().await {
                    match cmd {
                        UiToNet::Connect { target_id, password } => {
                            *target_for_ui.write().await = Some(target_id.clone());
                            let password_hash = password.map(|p| {
                                use sha2::{Digest, Sha256};
                                let mut hasher = Sha256::new();
                                hasher.update(b"RemoteDesktopSalt_");
                                hasher.update(p.as_bytes());
                                format!("{:x}", hasher.finalize())
                            });

                            let _ = outbound_for_ui.send(SignalMessage::ConnectRequest {
                                target_id,
                                password_hash,
                                auth_signature: None,
                            });
                        }

                        UiToNet::AcceptIncoming { from_id, permissions } => {
                            log::info!("✅ [SESIÓN INICIADA] Conexión aceptada para [{}]. Iniciando streaming...", from_id);
                            *active_ctrl_for_ui.write().await = Some(from_id.clone());
                            *active_perm_for_ui.write().await = permissions;
                            let _ = outbound_for_ui.send(SignalMessage::ConnectAccept {
                                from_id,
                                permissions,
                            });
                        }

                        UiToNet::RejectIncoming { from_id } => {
                            let _ = outbound_for_ui.send(SignalMessage::ConnectReject {
                                from_id,
                                reason: "El usuario remoto rechazó la sesión de WolfDesk.".to_string(),
                            });
                        }

                        UiToNet::SendChat { target_id, message } => {
                            let _ = outbound_for_ui.send(SignalMessage::Chat {
                                target_id,
                                text: message,
                            });
                        }

                        UiToNet::SendFile { target_id, file_path } => {
                            let tx_clone = outbound_for_ui.clone();
                            let tx_notify = tx_ui_notify_cmd.clone();
                            tokio::spawn(async move {
                                let path = Path::new(&file_path);
                                match FileTransferManager::send_file(path, &target_id, &tx_clone).await {
                                    Ok(msg) => {
                                        let _ = tx_notify.send(NetToUi::FileStatus(format!("✅ {}", msg)));
                                    }
                                    Err(err) => {
                                        let _ = tx_notify.send(NetToUi::FileStatus(format!("❌ Error al enviar archivo: {}", err)));
                                    }
                                }
                            });
                        }

                        UiToNet::RequestRemoteFiles { target_id, path } => {
                            let _ = outbound_for_ui.send(SignalMessage::FileListRequest {
                                target_id,
                                path,
                            });
                        }

                        UiToNet::DownloadRemoteFile { target_id, remote_file_path } => {
                            let _ = outbound_for_ui.send(SignalMessage::FileDownloadRequest {
                                target_id,
                                remote_file_path,
                            });
                        }

                        UiToNet::UpdateConfig(cfg) => {
                            *shared_cfg_for_ui.write().await = cfg;
                        }
                    }
                }
            });

            // Bucle principal para recibir mensajes desde el Servidor de Señalización
            let tx_ui_notify = tx_to_ui.clone();
            let active_ctrl_from_net = active_controller_id.clone();
            let active_perm_from_net = active_permissions.clone();
            let cfg_from_net = shared_config.clone();
            let outbound_auto_accept = outbound_tx.clone();
            let viewer_tx_clone = viewer_frame_tx.clone();
            let viewer_ctrl_tx_clone = viewer_control_tx.clone();
            let quality_on_host = active_streaming_quality.clone();
            let file_manager_clone = file_manager.clone();

            while let Some(msg) = inbound_rx.recv().await {
                match msg {
                    SignalMessage::RegisterSuccess { assigned_id } => {
                        let _ = tx_ui_notify.send(NetToUi::ServerConnected(assigned_id));
                    }

                    SignalMessage::ConnectRequest { target_id, password_hash, .. } => {
                        let cfg = cfg_from_net.read().await;
                        let mut auto_accept = false;
                        if let (Some(expected), Some(received)) = (&cfg.unattended_password_hash, &password_hash) {
                            if expected == received {
                                auto_accept = true;
                            }
                        }

                        if auto_accept {
                            log::info!("Acceso desatendido WolfDesk autorizado para [{}]", target_id);
                            *active_ctrl_from_net.write().await = Some(target_id.clone());
                            *active_perm_from_net.write().await = cfg.default_permissions;
                            let _ = outbound_auto_accept.send(SignalMessage::ConnectAccept {
                                from_id: target_id.clone(),
                                permissions: cfg.default_permissions,
                            });
                            let _ = tx_ui_notify.send(NetToUi::SessionAccepted(target_id));
                        } else {
                            log::info!("🔔 [SESIÓN ENTRANTE] Solicitud de conexión desde [{}]. Presione '🟢 ACEPTAR' en la pantalla de Linux.", target_id);
                            let _ = tx_ui_notify.send(NetToUi::IncomingRequest(target_id));
                        }
                    }

                    SignalMessage::ConnectAccept { from_id, .. } => {
                        *active_target_id.write().await = Some(from_id.clone());
                        let _ = tx_ui_notify.send(NetToUi::SessionAccepted(from_id.clone()));

                        let (f_tx, f_rx) = std::sync::mpsc::channel::<Vec<u8>>();
                        *viewer_tx_clone.lock().unwrap() = Some(f_tx);

                        let title = format!("🐺 WolfDesk - Sesión Activa con [{}]", from_id);
                        let ctrl_tx = viewer_ctrl_tx_clone.clone();
                        std::thread::spawn(move || {
                            start_viewer_window(&title, f_rx, ctrl_tx);
                        });
                    }

                    SignalMessage::ConnectReject { reason, .. } => {
                        let _ = tx_ui_notify.send(NetToUi::SessionRejected(reason));
                    }

                    SignalMessage::Chat { target_id: from_id, text } => {
                        let _ = tx_ui_notify.send(NetToUi::ChatReceived { from_id, text });
                    }

                    SignalMessage::FileTransferStart { file_name, file_size, total_chunks, .. } => {
                        file_manager_clone.lock().unwrap().handle_start(&file_name, file_size, total_chunks);
                        let _ = tx_ui_notify.send(NetToUi::FileStatus(format!("Recibiendo '{}' ({} KB)...", file_name, file_size / 1024)));
                    }

                    SignalMessage::FileTransferChunk { file_name, chunk_index, data_base64, .. } => {
                        file_manager_clone.lock().unwrap().handle_chunk(&file_name, chunk_index, &data_base64);
                    }

                    SignalMessage::FileTransferComplete { file_name, .. } => {
                        if let Some(saved_path) = file_manager_clone.lock().unwrap().handle_complete(&file_name) {
                            let _ = tx_ui_notify.send(NetToUi::FileStatus(format!("✅ ¡Archivo guardado en {:?}!", saved_path)));
                        }
                    }

                    SignalMessage::FileListRequest { target_id, path } => {
                        let (canon_path, entries) = FileTransferManager::list_directory(&path);
                        let _ = outbound_auto_accept.send(SignalMessage::FileListResponse {
                            target_id,
                            path: canon_path,
                            entries,
                        });
                    }

                    SignalMessage::FileListResponse { path, entries, .. } => {
                        let _ = tx_ui_notify.send(NetToUi::RemoteFilesReceived { path, entries });
                    }

                    SignalMessage::FileDownloadRequest { target_id, remote_file_path } => {
                        let tx_clone = outbound_auto_accept.clone();
                        let tx_notify = tx_ui_notify.clone();
                        tokio::spawn(async move {
                            let p = Path::new(&remote_file_path);
                            match FileTransferManager::send_file(p, &target_id, &tx_clone).await {
                                Ok(msg) => {
                                    let _ = tx_notify.send(NetToUi::FileStatus(format!("✅ {}", msg)));
                                }
                                Err(err) => {
                                    let _ = tx_notify.send(NetToUi::FileStatus(format!("❌ Error al enviar archivo: {}", err)));
                                }
                            }
                        });
                    }

                    SignalMessage::VideoFrame { jpeg_base64, .. } => {
                        use base64::Engine;
                        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(jpeg_base64) {
                            if let Some(ref s) = *viewer_tx_clone.lock().unwrap() {
                                let _ = s.send(bytes);
                            }
                        }
                    }

                    SignalMessage::Control { event, .. } => {
                        match &event {
                            ControlEvent::SetQuality { quality } => {
                                *quality_on_host.write().await = *quality;
                                println!(">> [WOLFDESK HOST] Calidad de captura ajustada dinámicamente a {}%", quality);
                            }
                            _ => {
                                static LOG_CTRL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                                if !LOG_CTRL.swap(true, std::sync::atomic::Ordering::Relaxed) {
                                    log::info!("🎮 [WOLFDESK HOST] Recibiendo eventos de control remoto desde el visor...");
                                }
                                let perms = *active_perm_from_net.read().await;
                                dispatch_event_with_permissions(&event, &perms);
                            }
                        }
                    }

                    SignalMessage::Error { message } => {
                        let _ = tx_ui_notify.send(NetToUi::SessionError(message));
                    }

                    _ => {}
                }
            }
        });
    });

    // 2. Iniciar la Interfaz Gráfica Nativa de WolfDesk con su Icono Oficial
    let mut viewport = ViewportBuilder::default()
        .with_inner_size(Vec2::new(820.0, 520.0))
        .with_min_inner_size(Vec2::new(700.0, 460.0))
        .with_title("WolfDesk Pro - Escritorio Remoto");

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let _res = eframe::run_native(
        "WolfDesk",
        native_options,
        Box::new(move |_cc| {
            Box::new(DashboardApp::new(
                net_identity_id,
                config,
                tx_to_net,
                rx_from_net,
            ))
        }),
    );

    std::process::exit(0);
}
