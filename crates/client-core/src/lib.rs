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
pub mod viewer;

pub use capture::{ScreenCapturer, ScreenFrame};
pub use config::AppConfig;
pub use dashboard::DashboardApp;
pub use file_transfer::FileTransferManager;
pub use identity_store::load_or_create_identity;
pub use input::dispatch_event_with_permissions;
pub use installer::{install_to_system, is_installed, uninstall_from_system};
pub use recorder::SessionRecorder;
pub use signaling_client::SignalingClient;
pub use terminal_log::{add_log, get_logs, init_terminal_logger, LogLevel};
pub use viewer::start_viewer_window;

