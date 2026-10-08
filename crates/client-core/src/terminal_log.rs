use std::sync::Mutex;
use std::time::SystemTime;

#[derive(Clone, PartialEq, Debug)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Success,
}

impl LogLevel {
    pub fn badge(&self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Success => "OK",
        }
    }
}

#[derive(Clone, Debug)]
pub struct LogMessage {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
}

static LOGS: Mutex<Vec<LogMessage>> = Mutex::new(Vec::new());

fn current_time_str() -> String {
    // Generar formato HH:MM:SS usando SystemTime estándar
    if let Ok(duration) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        let total_secs = duration.as_secs();
        let seconds = total_secs % 60;
        let minutes = (total_secs / 60) % 60;
        let hours = (total_secs / 3600) % 24;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        "00:00:00".to_string()
    }
}

pub fn add_log(level: LogLevel, message: &str) {
    if let Ok(mut logs) = LOGS.lock() {
        logs.push(LogMessage {
            timestamp: current_time_str(),
            level,
            message: message.to_string(),
        });
        if logs.len() > 1000 {
            logs.remove(0);
        }
    }
}

pub fn get_logs() -> Vec<LogMessage> {
    if let Ok(logs) = LOGS.lock() {
        logs.clone()
    } else {
        Vec::new()
    }
}

pub fn clear_logs() {
    if let Ok(mut logs) = LOGS.lock() {
        logs.clear();
    }
}

pub fn export_logs_string() -> String {
    if let Ok(logs) = LOGS.lock() {
        logs.iter()
            .map(|l| format!("[{}] [{}] {}", l.timestamp, l.level.badge(), l.message))
            .collect::<Vec<_>>()
            .join("\r\n")
    } else {
        String::new()
    }
}

/// Logger global para capturar logs de Rust y canalizarlos a la terminal GUI
pub struct AppLogger;

impl log::Log for AppLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let level = match record.level() {
                log::Level::Error => LogLevel::Error,
                log::Level::Warn => LogLevel::Warn,
                _ => LogLevel::Info,
            };
            let time_str = current_time_str();
            let msg = format!("{}", record.args());
            eprintln!("[{}] [{}] {}", time_str, level.badge(), msg);
            add_log(level, &msg);
        }
    }

    fn flush(&self) {}
}

pub fn init_terminal_logger() {
    static LOGGER: AppLogger = AppLogger;
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Info);
    add_log(LogLevel::Success, "Consola y terminal WolfDesk inicializada correctamente.");
}
