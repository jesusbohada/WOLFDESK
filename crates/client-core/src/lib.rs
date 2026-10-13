pub mod app_launcher;
pub mod brand;
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

pub use app_launcher::run_desktop_app;
pub use brand::{get_brand, BrandConfig, BrandType};
pub use capture::{enumerate_monitors, MonitorBounds, ScreenCapturer, ScreenFrame};
pub use config::{AppConfig, Contact};
pub use dashboard::DashboardApp;
pub use file_transfer::FileTransferManager;
pub use identity_store::load_or_create_identity;
pub use input::{dispatch_event_with_permissions, set_active_capture_bounds};
pub use installer::{install_to_system, is_installed, uninstall_from_system};
pub use recorder::SessionRecorder;
pub use signaling_client::SignalingClient;
pub use terminal_log::{add_log, get_logs, init_terminal_logger, LogLevel};
pub use tray::start_system_tray;
pub use updater::{check_for_updates, get_build_git_hash, get_local_version, perform_update, UpdateStatus};
pub use viewer::start_viewer_window;

/// Reinicia completamente la aplicación / servicio después de una breve pausa
pub fn restart_application() {
    let brand = get_brand();
    log::info!("🔄 [RESTART] Reiniciando la aplicación / servicio {} de forma limpia...", brand.app_name);

    #[cfg(target_os = "linux")]
    {
        let bin_name = brand.unix_bin_name;
        let exe = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from(format!("/opt/{}/{}", bin_name, bin_name)));
        let exe_str = exe.to_string_lossy().to_string();
        let display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
        let xauthority = std::env::var("XAUTHORITY").unwrap_or_else(|_| "/home/caja/.Xauthority".to_string());

        // 1. Si existe el servicio systemd, solicitar reinicio formal a systemd
        let service_file = format!("/etc/systemd/system/{}.service", bin_name);
        if std::path::Path::new(&service_file).exists() {
            let _ = std::process::Command::new("systemctl")
                .args(["restart", bin_name])
                .spawn();
        } else {
            // 2. Si no es servicio systemd, esperar salida, limpiar procesos y relanzar
            let restart_cmd = format!(
                "sleep 1 && killall -9 {} 2>/dev/null; sleep 0.5 && DISPLAY={} XAUTHORITY={} nohup '{}' >/dev/null 2>&1 &",
                bin_name, display, xauthority, exe_str
            );
            let _ = std::process::Command::new("sh")
                .args(["-c", &restart_cmd])
                .spawn();
        }
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;

        if let Ok(mut exe) = std::env::current_exe() {
            let official_exe = brand.exe_name;

            // Si el ejecutable actual fue renombrado durante la actualización a *.old_*,
            // restauramos el nombre oficial para no relanzar el binario antiguo
            if let Some(file_name) = exe.file_name().and_then(|n| n.to_str()) {
                if file_name.contains(".old") {
                    exe.set_file_name(official_exe);
                }
            }
            let exe_str = exe.to_string_lossy().to_string();
            let parent_dir = exe.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();

            let current_pid = std::process::id();
            // Usar ping 127.0.0.1 -n 2 para una pausa no bloqueante de 1s (timeout falla sin consola adjunta).
            // Si el proceso actual no cerró limpiamente tras 1s, se asegura matando únicamente su PID (sin /T).
            // Luego lanza el nuevo ejecutable completamente desvinculado.
            let batch_cmd = if !parent_dir.is_empty() {
                format!(
                    "ping 127.0.0.1 -n 2 >nul & taskkill /F /PID {} >nul 2>&1 & cd /d \"{}\" & start \"\" \"{}\"",
                    current_pid, parent_dir, exe_str
                )
            } else {
                format!(
                    "ping 127.0.0.1 -n 2 >nul & taskkill /F /PID {} >nul 2>&1 & start \"\" \"{}\"",
                    current_pid, exe_str
                )
            };

            let _ = std::process::Command::new("cmd")
                .args(["/C", &batch_cmd])
                .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
                .spawn();
        }
    }

    std::process::exit(0);
}

/// Finaliza y mata completamente la aplicación y todos sus procesos,
/// liberando sockets, archivos en disco y el ejecutable bloqueado por el sistema operativo.
pub fn terminate_application() {
    let brand = get_brand();
    log::info!("🛑 [TERMINATE] Cerrando y matando procesos de {} para liberar recursos...", brand.app_name);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;

        let process_pattern = format!("{}.exe", brand.unix_bin_name);
        let kill_cmd = format!("ping 127.0.0.1 -n 2 >nul & taskkill /F /IM {} /T >nul 2>&1", process_pattern);
        let _ = std::process::Command::new("cmd")
            .args(["/C", &kill_cmd])
            .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
            .spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("killall")
            .args(["-9", brand.unix_bin_name])
            .spawn();
    }

    std::process::exit(0);
}

