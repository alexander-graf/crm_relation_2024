use eframe::egui;
use std::path::PathBuf;
mod app;
mod db;
mod ui;
pub mod config;

fn load_initial_config() {
    let config_path = PathBuf::from(format!("{}/.config/zugangsdaten.ini", std::env::var("HOME").unwrap()));
    if config_path.exists() {
        match ui::load_config_from_file(&config_path) {
            Ok(config) => {
                db::set_config(Some(config));
                println!("Existing configuration loaded.");
            }
            Err(e) => {
                eprintln!("Error loading configuration: {}", e);
            }
        }
    } else {
        println!("No configuration file found at {:?}", config_path);
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    load_initial_config();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "CRM Application",
        options,
        Box::new(|cc| Box::new(app::CrmApp::new(cc))),
    )
}
