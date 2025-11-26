//! Settings component

use eframe::egui::{self, Context, Id, Modal};

use crate::app::{Bladvak, BladvakApp};

/// Settings object
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    /// Is setting modal open
    pub open: bool,

    /// Minimum width for the sidebar
    pub min_width_sidebar: f32,

    /// Right panel toggle
    pub right_panel: bool,

    /// Debug and inspection toggle
    pub show_inspection: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_inspection: false,
            open: false,
            min_width_sidebar: 200.0,
            right_panel: true,
        }
    }
}

impl<M> Bladvak<M>
where
    M: BladvakApp,
{
    /// Show the error manager ui
    pub fn show_error_manager(&mut self, ctx: &Context) {
        if !self.error_manager.was_open && !self.error_manager.errors.is_empty() {
            self.error_manager.is_open = true;
        }
        egui::Window::new("Errors")
            .open(&mut self.error_manager.is_open)
            .vscroll(true)
            .show(ctx, |ui| {
                for error in &self.error_manager.errors {
                    ui.label(error.message.clone());
                }
            });
        if !self.error_manager.is_open {
            self.error_manager.errors.clear();
        }
        self.error_manager.was_open = self.error_manager.is_open;
    }

    /// Show settings Ui
    pub fn show_setting(&mut self, ctx: &Context) {
        egui::Window::new("Inspection")
            .open(&mut self.settings.show_inspection)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.inspection_ui(ui);
            });
        if self.settings.open {
            let modal = Modal::new(Id::new("Modal settings")).show(ctx, |ui| {
                ui.label(format!("{} settings", M::name()));
                ui.separator();
                ui.checkbox(&mut self.settings.show_inspection, "Debug panel");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(format!("{} settings", self.error_manager.title()));
                    ui.button("⟳").clicked().then(|| {
                        self.error_manager = Default::default();
                    });
                });
                ui.checkbox(&mut self.error_manager.is_open, "Error panel");
                self.app.settings(ui, &mut self.error_manager);
                egui::Sides::new().show(
                    ui,
                    |_ui| {},
                    |modal_ui| {
                        if modal_ui.button("Close").clicked() {
                            modal_ui.close();
                        }
                    },
                );
            });
            if modal.should_close() {
                self.settings.open = false;
            }
        }
    }
}
