use eframe::egui;
use crate::ui;
use crate::db::{self, Customer};
use std::sync::{Arc, Mutex};

pub struct CrmApp {
    current_view: View,
    customers: Arc<Mutex<Vec<Customer>>>,
}

#[derive(PartialEq)]
pub enum View {
    Main,
    SetupWizard,
    Customers,
    Invoices,
    Settings,
}

impl Default for CrmApp {
    fn default() -> Self {
        Self {
            current_view: View::Main,
            customers: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl CrmApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            current_view: View::Main,
            customers: Arc::new(Mutex::new(Vec::new())),
        }
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
        ui::render_menu_bar(ctx, &mut self.current_view);

        match self.current_view {
            View::Main => ui::render_main_view(ctx),
            View::Customers => ui::render_customers_view(ctx, self.customers.clone()),
            View::Invoices => ui::render_invoices_view(ctx),
            View::Settings => ui::render_settings_view(ctx),
            View::SetupWizard => ui::render_setup_wizard_view(ctx),
        }

        // Load customers when switching to the Customers view
        if self.current_view == View::Customers {
            self.load_customers();
        }
    }
}

