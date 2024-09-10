use eframe::egui;
use crate::ui;

pub struct CrmApp {
    current_view: View,
}

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
        }
    }
}

impl CrmApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for CrmApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::render_menu_bar(ctx, &mut self.current_view);

        match self.current_view {
            View::Main => ui::render_main_view(ctx),
            View::SetupWizard => ui::render_setup_wizard_view(ctx),
            View::Customers => ui::render_customers_view(ctx),
            View::Invoices => ui::render_invoices_view(ctx),
            View::Settings => ui::render_settings_view(ctx),
        }
    }
}
