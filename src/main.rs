use eframe::egui;
mod app;
mod db;
mod ui;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
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
