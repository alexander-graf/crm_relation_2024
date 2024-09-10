use eframe::egui;
use crate::app::View;
use crate::db::{self, DbConfig};
use std::path::PathBuf;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::fs;
use serde_json;

static STEP: Lazy<Mutex<u8>> = Lazy::new(|| Mutex::new(1));
static DB_CONFIG: Lazy<Mutex<Option<DbConfig>>> = Lazy::new(|| Mutex::new(None));

pub fn render_menu_bar(ctx: &egui::Context, current_view: &mut View) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Setup Wizard").clicked() {
                    *current_view = View::SetupWizard;
                }
                if ui.button("Settings").clicked() {
                    *current_view = View::Settings;
                }
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            });

            ui.menu_button("View", |ui| {
                if ui.button("Customers").clicked() {
                    *current_view = View::Customers;
                }
                if ui.button("Invoices").clicked() {
                    *current_view = View::Invoices;
                }
            });

            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    // TODO: Implement About dialog
                }
            });
        });
    });
}

pub fn render_main_view(ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Welcome to CRM Application");
        ui.label("Select an option from the menu to get started.");
    });
}

pub fn render_setup_wizard_view(ctx: &egui::Context) {
    egui::Window::new("Setup Wizard")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            let mut step = STEP.lock().unwrap();
            
            if *step == 1 {
                let config_path = PathBuf::from(format!("{}/.config/zugangsdaten.ini", std::env::var("HOME").unwrap()));
                if config_path.exists() {
                    match load_config_from_file(&config_path) {
                        Ok(config) => {
                            *DB_CONFIG.lock().unwrap() = Some(config);
                            ui.label("Existing configuration loaded.");
                        }
                        Err(e) => {
                            ui.label(format!("Error loading configuration: {}", e));
                        }
                    }
                }
            }
            
            match *step {
                1 => render_step_one(ui),
                2 => render_step_two(ui),
                _ => {
                    ui.label("Setup complete!");
                    if ui.button("Close").clicked() {
                        *step = 1;
                        *DB_CONFIG.lock().unwrap() = None;
                    }
                }
            }
        });
}

fn render_step_one(ui: &mut egui::Ui) {
    ui.heading("Step 1: Database Configuration");
    
    let mut db_config = DB_CONFIG.lock().unwrap();
    let config = db_config.get_or_insert_with(DbConfig::default);
    
    ui.horizontal(|ui| {
        ui.label("Host:");
        ui.text_edit_singleline(&mut config.host);
    });
    ui.horizontal(|ui| {
        ui.label("Port:");
        ui.text_edit_singleline(&mut config.port);
    });
    ui.horizontal(|ui| {
        ui.label("Username:");
        ui.text_edit_singleline(&mut config.username);
    });
    ui.horizontal(|ui| {
        ui.label("Password:");
        ui.add(egui::TextEdit::singleline(&mut config.password).password(true));
    });
    ui.horizontal(|ui| {
        ui.label("Database Name:");
        ui.text_edit_singleline(&mut config.database);
    });

    if ui.button("Next").clicked() {
        let config_path = PathBuf::from(format!("{}/.config/zugangsdaten.ini", std::env::var("HOME").unwrap()));
        ui.label(format!("Saving config to: {:?}", config_path));
        
        if let Err(e) = save_config_to_file(&config, &config_path) {
            ui.label(format!("Error saving configuration: {}", e));
        } else {
            ui.label("Configuration saved successfully.");
        }

        let config_clone = config.clone();
        tokio::spawn(async move {
            match db::create_database(&config_clone).await {
                Ok(_) => {
                    println!("Database created successfully!");
                    *STEP.lock().unwrap() = 2;
                },
                Err(e) => {
                    eprintln!("Error creating database: {}", e);
                }
            }
        });
    }
}

fn render_step_two(ui: &mut egui::Ui) {
    ui.heading("Step 2: Create Database Structure");
    
    if ui.button("Create Database Structure").clicked() {
        if let Some(config) = DB_CONFIG.lock().unwrap().clone() {
            ui.label(format!("Attempting to create database structure for: {}", config.database));
            tokio::spawn(async move {
                match db::create_database_structure(&config).await {
                    Ok(_) => {
                        println!("Database structure created successfully!");
                        *STEP.lock().unwrap() = 3;
                    },
                    Err(e) => {
                        eprintln!("Error creating database structure: {}", e);
                    }
                }
            });
        } else {
            ui.label("No database configuration found!");
        }
    }
}

pub fn render_customers_view(ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Customers");
        ui.label("Here you can manage your customers.");
        if ui.button("Add New Customer").clicked() {
            // TODO: Implement add new customer functionality
        }
        if ui.button("View Customer List").clicked() {
            // TODO: Implement view customer list functionality
        }
    });
}

pub fn render_invoices_view(ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Invoices");
        ui.label("Here you can manage your invoices.");
        if ui.button("Create New Invoice").clicked() {
            // TODO: Implement create new invoice functionality
        }
        if ui.button("View Invoice List").clicked() {
            // TODO: Implement view invoice list functionality
        }
    });
}

pub fn render_settings_view(ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Settings");
        ui.label("Configure your CRM application here.");
        if ui.button("Database Settings").clicked() {
            // TODO: Implement database settings functionality
        }
        if ui.button("User Preferences").clicked() {
            // TODO: Implement user preferences functionality
        }
    });
}

fn save_config_to_file(config: &DbConfig, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let config_json = serde_json::to_string_pretty(config)?;
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, config_json)?;
    Ok(())
}

fn load_config_from_file(path: &PathBuf) -> Result<DbConfig, Box<dyn std::error::Error>> {
    let config_json = fs::read_to_string(path)?;
    let config: DbConfig = serde_json::from_str(&config_json)?;
    Ok(config)
}
