pub mod capture;
pub mod config;
pub mod dashboard;
pub mod file_transfer;
pub mod identity_store;
pub mod input;
pub mod installer;
pub mod recorder;
pub mod signaling_client;
pub mod terminal_log;
pub mod tray;
pub mod updater;
pub mod viewer;

pub use capture::{ScreenCapturer, ScreenFrame};
pub use config::{AppConfig, Contact};
pub use dashboard::DashboardApp;
pub use file_transfer::FileTransferManager;
pub use identity_store::load_or_create_identity;
pub use input::dispatch_event_with_permissions;
pub use installer::{install_to_system, is_installed, uninstall_from_system};
pub use recorder::SessionRecorder;
pub use signaling_client::SignalingClient;
pub use terminal_log::{add_log, get_logs, init_terminal_logger, LogLevel};
pub use tray::start_system_tray;
pub use updater::{check_for_updates, get_build_git_hash, get_local_version, perform_update, UpdateStatus};
pub use viewer::start_viewer_window;

/// Reinicia completamente la aplicación / servicio WolfDesk después de una breve pausa
pub fn restart_application() {
    log::info!("🐺 [RESTART] Reiniciando la aplicación / servicio WolfDesk en 2 segundos...");

    #[cfg(target_os = "linux")]
    {
        let exe = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("/opt/wolfdesk/wolfdesk"));
        let exe_str = exe.to_string_lossy().to_string();
        let display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
        let xauthority = std::env::var("XAUTHORITY").unwrap_or_else(|_| "/home/caja/.Xauthority".to_string());

        // 1. Si existe el servicio systemd, solicitar reinicio a systemd
        if std::path::Path::new("/etc/systemd/system/wolfdesk.service").exists() {
            let _ = std::process::Command::new("systemctl")
                .args(["restart", "wolfdesk"])
                .spawn();
        } else {
            // 2. Si no es servicio systemd, lanzar proceso en segundo plano que espere 2s y vuelva a abrir WolfDesk
            let restart_cmd = format!(
                "sleep 2 && DISPLAY={} XAUTHORITY={} nohup '{}' >/dev/null 2>&1 &",
                display, xauthority, exe_str
            );
            let _ = std::process::Command::new("sh")
                .args(["-c", &restart_cmd])
                .spawn();
        }
    }

    #[cfg(windows)]
    {
        if let Ok(mut exe) = std::env::current_exe() {
            // Si el ejecutable actual fue renombrado durante la actualización a *.old_*,
            // debemos restaurar el nombre original wolfdesk.exe para no relanzar el binario antiguo
            if let Some(file_name) = exe.file_name().and_then(|n| n.to_str()) {
                if file_name.contains(".old") {
                    exe.set_file_name("wolfdesk.exe");
                }
            }
            let exe_str = exe.to_string_lossy().to_string();
            let cmd = format!("Start-Sleep -Seconds 2; Start-Process -FilePath '{}'", exe_str);
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &cmd])
                .spawn();
        }
    }

    std::process::exit(0);
}

