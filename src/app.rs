use crate::db::{self, ContactHistory, Customer};
use crate::ui;
use chrono::{NaiveDate, Utc};

use eframe::egui;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::config;
use std::env;
use std::path::PathBuf;

pub struct CrmApp {
    current_view: View,
    customers: Arc<Mutex<Vec<Customer>>>,
    customer_contact_window_open: bool,
    active_customer_index: usize,
    contact_history_cache: Arc<Mutex<HashMap<i32, Vec<ContactHistory>>>>,
    search_query: String,  // für CustomerSearch
    search_results: Vec<Customer>,
    selected_customer: Option<Customer>,
    new_contact_history: ContactHistory,
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
    CustomerSearch, // Neuer Menüpunkt
}

impl Default for CrmApp {
    fn default() -> Self {
        Self {
            current_view: View::Main,
            customers: Arc::new(Mutex::new(Vec::new())),
            customer_contact_window_open: false,
            active_customer_index: 0,
            contact_history_cache: Arc::new(Mutex::new(HashMap::new())),
            search_query: String::new(),
            search_results: Vec::new(),
            selected_customer: None,
            new_contact_history: ContactHistory::default(),
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
            search_query: String::new(),
            search_results: Vec::new(),
            selected_customer: None,
            new_contact_history: ContactHistory::default(),
        };

        // Load the database configuration
        let config_path = PathBuf::from(format!(
            "{}/.config/zugangsdaten.ini",
            env::var("HOME").unwrap()
        ));
        let db_config =
            config::load_db_config(&config_path).expect("Failed to load database configuration");

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

    fn render_customer_search(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            if ui.text_edit_singleline(&mut self.search_query).changed() {
                self.search_customers();
            }
        });
    
        egui::ScrollArea::vertical().show(ui, |ui| {
            for customer in &self.search_results {
                if ui.button(&customer.contact_name).double_clicked() {
                    self.selected_customer = Some(customer.clone());
                    self.new_contact_history = ContactHistory::default();
                    self.new_contact_history.customer_id = customer.customer_id;
                }
            }
        });
    
        if let Some(customer) = &self.selected_customer {
            ui.group(|ui| {
                ui.label(format!("New Contact History for {}", customer.contact_name));
                ui.horizontal(|ui| {
                    ui.label("Contact Type:");
                    ui.text_edit_singleline(&mut self.new_contact_history.contact_type);
                });
                ui.horizontal(|ui| {
                    ui.label("Contact Date:");
                    let mut date_string = self.new_contact_history.contact_date.format("%Y-%m-%d").to_string();
                    if ui.text_edit_singleline(&mut date_string).changed() {
                        if let Ok(date) = NaiveDate::parse_from_str(&date_string, "%Y-%m-%d") {
                            if let Some(datetime) = date.and_hms_opt(0, 0, 0) {
                                self.new_contact_history.contact_date = datetime.and_local_timezone(Utc).unwrap();
                            }
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Notes:");
                    ui.text_edit_multiline(&mut self.new_contact_history.notes);
                });
            });
    
            // Move this outside of the ui.group closure
            if ui.button("Save").clicked() {
                if !self.new_contact_history.contact_type.is_empty() && !self.new_contact_history.notes.is_empty() {
                    self.save_contact_history();
                } else {
                    ui.label("Please fill in all required fields");
                }
            }
        }
    }
    

    fn search_customers(&mut self) {
        let search_query = self.search_query.clone();
        let customers = Arc::clone(&self.customers);
        let search_results = Arc::new(Mutex::new(Vec::new()));

        let search_results_clone = Arc::clone(&search_results);
        tokio::spawn(async move {
            let customers = customers.lock().unwrap();
            let results: Vec<Customer> = customers
                .iter()
                .filter(|c| c.contact_name.to_lowercase().contains(&search_query.to_lowercase()))
                .cloned()
                .collect();
            *search_results_clone.lock().unwrap() = results;
        });

        self.search_results = search_results.lock().unwrap().clone();
    }

    fn save_contact_history(&mut self) {
        if let Some(config) = db::get_config() {
            let new_history = self.new_contact_history.clone();
            tokio::spawn(async move {
                match db::add_contact_history(&config, &new_history).await {
                    Ok(_) => println!("Contact history saved successfully"),
                    Err(e) => eprintln!("Error saving contact history: {}", e),
                }
            });
        }
    }

    fn ensure_customers_loaded(&self) -> bool {
        let customers = self.customers.lock().unwrap();
        if customers.is_empty() {
            // If no customers are loaded, try to load them again
            drop(customers); // Release the lock
            let customers = Arc::clone(&self.customers);
            let config_path = PathBuf::from(format!(
                "{}/.config/zugangsdaten.ini",
                env::var("HOME").unwrap()
            ));
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
            ui.label(format!(
                "Customer {} of {}",
                self.active_customer_index + 1,
                customer_count
            ));
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
            println!(
                "Rendering history for customer {}: {} entries",
                customer.customer_id,
                history.len()
            );
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
                    println!(
                        "Loaded {} history entries for customer {}",
                        history.len(),
                        customer_id
                    );
                    history_cache.lock().unwrap().insert(customer_id, history);
                }
                Err(e) => eprintln!(
                    "Error loading contact history for customer {}: {}",
                    customer_id, e
                ),
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
                    }
                    Err(e) => eprintln!("Error fetching customers: {}", e),
                }
            }
        });
    }
}


impl eframe::App for CrmApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::render_menu_bar(
            ctx,
            &mut self.current_view,
            &mut self.customer_contact_window_open,
        );

        match self.current_view {
            View::Main => ui::render_main_view(ctx),
            View::Customers => {
                let customers = self.customers.clone();
                ui::render_customers_view(ctx, customers);
            }
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
            View::CustomerSearch => {
                egui::Window::new("Customer Search")
                    .show(ctx, |ui| {
                        self.render_customer_search(ui);
                    });
            },
        }

        // Load customers when switching to the Customers view
        if self.current_view == View::Customers {
            self.load_customers();
        }
    }
}


