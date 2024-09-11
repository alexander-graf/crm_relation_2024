use eframe::egui;
use crate::ui;
use crate::db::{self, ContactHistory, Customer};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::config;
use std::path::PathBuf;
use std::env;



pub struct CrmApp {
    current_view: View,
    customers: Arc<Mutex<Vec<Customer>>>,
    customer_contact_window_open: bool,
    active_customer_index: usize,
    contact_history_cache: Arc<Mutex<HashMap<i32, Vec<ContactHistory>>>>,
}

// Menüpunkte
#[derive(PartialEq)]
pub enum View {
    Main,
    SetupWizard,
    Customers,
    Invoices,
    Settings,
    CustomerContact,
}

impl Default for CrmApp {
    fn default() -> Self {
        Self {
            current_view: View::Main,
            customers: Arc::new(Mutex::new(Vec::new())),
            customer_contact_window_open: false,
            active_customer_index: 0,
            contact_history_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl CrmApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let app = Self {
            current_view: View::Main,
            customers: Arc::new(Mutex::new(Vec::new())),
            customer_contact_window_open: false,
            active_customer_index: 0,
            contact_history_cache: Arc::new(Mutex::new(HashMap::new())),
        };

        // Load the database configuration
        let config_path = PathBuf::from(format!("{}/.config/zugangsdaten.ini", env::var("HOME").unwrap()));
        let db_config = config::load_db_config(&config_path).expect("Failed to load database configuration");

        // Load customers asynchronously
        let customers = Arc::clone(&app.customers);
        let db_config_clone = db_config.clone();
        tokio::spawn(async move {
            if let Ok(loaded_customers) = db::get_customers(&db_config_clone).await {
                *customers.lock().unwrap() = loaded_customers;
            }
        });

        app
    }

    fn ensure_customers_loaded(&self) -> bool {
        let customers = self.customers.lock().unwrap();
        if customers.is_empty() {
            // If no customers are loaded, try to load them again
            drop(customers); // Release the lock
            let customers = Arc::clone(&self.customers);
            let config_path = PathBuf::from(format!("{}/.config/zugangsdaten.ini", env::var("HOME").unwrap()));
            tokio::spawn(async move {
                match config::load_db_config(&config_path) {
                    Ok(db_config) => {
                        if let Ok(loaded_customers) = db::get_customers(&db_config).await {
                            *customers.lock().unwrap() = loaded_customers;
                        }
                    }
                    Err(e) => eprintln!("Failed to load database configuration: {}", e),
                }
            });
            false
        } else {
            true
        }
    }
    
fn render_customer_contact(&mut self, ui: &mut egui::Ui) {
    if !self.ensure_customers_loaded() {
        ui.label("Loading customer data...");
        return;
    }
    let customer_count = self.customers.lock().unwrap().len();
    if customer_count == 0 {
        ui.label("No customer records available.");
        return;
    }

    let customer_id = {
        let customers = self.customers.lock().unwrap();
        customers[self.active_customer_index].customer_id
    };

    ui.horizontal(|ui| {
        if ui.button("< Previous").clicked() && self.active_customer_index > 0 {
            self.active_customer_index -= 1;
            self.load_contact_history(customer_id);
        }
        ui.label(format!("Customer {} of {}", self.active_customer_index + 1, customer_count));
        if ui.button("Next >").clicked() && self.active_customer_index < customer_count - 1 {
            self.active_customer_index += 1;
            self.load_contact_history(customer_id);
        }
    });

    ui.add_space(20.0);

    // Customer form fields
    let mut customer = self.customers.lock().unwrap()[self.active_customer_index].clone();

    ui.horizontal(|ui| {
        ui.label("Company Name:");
        ui.text_edit_singleline(&mut customer.company_name);
    });

    ui.horizontal(|ui| {
        ui.label("Contact Name:");
        ui.text_edit_singleline(&mut customer.contact_name.to_string());
    });

    ui.horizontal(|ui| {
        ui.label("Email:");
        ui.text_edit_singleline(&mut customer.email.to_string());
    });

    ui.horizontal(|ui| {
        ui.label("Phone:");
        ui.text_edit_singleline(&mut customer.phone.to_string());
    });

    ui.horizontal(|ui| {
        ui.label("Address:");
        ui.text_edit_multiline(&mut customer.address.to_string());
    });

    ui.horizontal(|ui| {
        ui.label("City:");
        ui.text_edit_singleline(&mut customer.city.to_string());
    });

    ui.horizontal(|ui| {
        ui.label("Postal Code:");
        ui.text_edit_singleline(&mut customer.postal_code.to_string());
    });

    ui.horizontal(|ui| {
        ui.label("Country:");
        ui.text_edit_singleline(&mut customer.country.to_string());
    });

    // Contact History
    ui.add_space(20.0);
    ui.heading("Contact History");

    let history = self.contact_history_cache.lock().unwrap();
    if let Some(history) = history.get(&customer.customer_id) {
        println!("Rendering history for customer {}: {} entries", customer.customer_id, history.len());
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("contact_history_grid").show(ui, |ui| {
                ui.label("Date");
                ui.label("Type");
                ui.label("Method");
                ui.label("Outcome");
                ui.label("Notes");
                ui.end_row();

                for entry in history {
                    ui.label(entry.contact_date.format("%Y-%m-%d %H:%M").to_string());
                    ui.label(&entry.contact_type);
                    ui.label(entry.contact_method.as_deref().unwrap_or("-"));
                    ui.label(&entry.contact_outcome);
                    ui.label(&entry.notes);
                    ui.end_row();
                }
            });
        });
    } else {
        ui.label("Loading contact history...");
        let customer_id = customer.customer_id;
        self.load_contact_history(customer_id);
    }
}

fn load_contact_history(&self, customer_id: i32) {
    let config = db::get_config().unwrap();
    let history_cache = Arc::clone(&self.contact_history_cache);
    tokio::spawn(async move {
        match db::get_contact_history(&config, customer_id).await {
            Ok(history) => {
                println!("Loaded {} history entries for customer {}", history.len(), customer_id);
                history_cache.lock().unwrap().insert(customer_id, history);
            }
            Err(e) => eprintln!("Error loading contact history for customer {}: {}", customer_id, e),
        }
    });
}


    pub fn load_customers(&self) {
        let customers_clone = self.customers.clone();
        tokio::spawn(async move {
            if let Some(config) = db::get_config() {
                match db::get_customers(&config).await {
                    Ok(fetched_customers) => {
                        let mut customers = customers_clone.lock().unwrap();
                        *customers = fetched_customers;
                    },
                    Err(e) => eprintln!("Error fetching customers: {}", e),
                }
            }
        });
    }
}

impl eframe::App for CrmApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::render_menu_bar(ctx, &mut self.current_view, &mut self.customer_contact_window_open);

        match self.current_view {
            View::Main => ui::render_main_view(ctx),
            View::Customers => {
                let customers = self.customers.clone();
                ui::render_customers_view(ctx, customers);
            },
            View::Invoices => ui::render_invoices_view(ctx),
            View::Settings => ui::render_settings_view(ctx),
            View::SetupWizard => ui::render_setup_wizard_view(ctx),
            View::CustomerContact => {
                if self.customer_contact_window_open {
                    let mut open = self.customer_contact_window_open;
                    egui::Window::new("Customer Contact")
                        .open(&mut open)
                        .show(ctx, |ui| {
                            self.render_customer_contact(ui);
                        });
                    self.customer_contact_window_open = open;
                }
            },
        }

        // Load customers when switching to the Customers view
        if self.current_view == View::Customers {
            self.load_customers();
        }
    }
}
