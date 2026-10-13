#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> Result<(), eframe::Error> {
    client_core::run_desktop_app()
}
