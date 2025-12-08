//! Settings component

use eframe::egui::{self, Context, Id, Modal};
use serde::{Deserialize, Serialize};

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
    M: BladvakApp + Serialize + for<'a> Deserialize<'a> + 'static,
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
    pub fn show_setting(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        egui::Window::new("Inspection")
            .open(&mut self.settings.show_inspection)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.inspection_ui(ui);
            });
        if self.settings.open {
            let modal = Modal::new(Id::new("Modal settings")).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                ui.label(format!("{}@{} settings", M::name(), M::version()));
                            });
                            ui.add(
                                egui::Hyperlink::from_label_and_url("repository", M::repo_url())
                                    .open_in_new_tab(true),
                            );
                        });
                    });
                    ui.button("⟳").clicked().then(|| {
                        if let Some(storage) = frame.storage_mut() {
                            eframe::set_value(storage, eframe::APP_KEY, self);
                            log::info!("Storage reset");
                        }
                    });
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(format!("{} settings", self.error_manager.title()));
                    ui.button("⟳").clicked().then(|| {
                        self.error_manager = Default::default();
                    });
                });
                ui.checkbox(&mut self.error_manager.is_open, "Error panel");
                self.app.settings(ui, &mut self.error_manager);
                ui.separator();
                egui::Sides::new().spacing(20.0).show(
                    ui,
                    |modal_ui| {
                        modal_ui.vertical(|ui| {
                            ui.checkbox(&mut self.settings.show_inspection, "Debug panel");
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                ui.label("Using ");
                                ui.add(
                                    egui::Hyperlink::from_label_and_url(
                                        concat!("bladvak", env!("CARGO_PKG_VERSION")),
                                        "https://github.com/Its-Just-Nans/bladvak",
                                    )
                                    .open_in_new_tab(true),
                                );
                            });
                        });
                    },
                    |modal_ui| {
                        modal_ui.horizontal_centered(|ui| {
                            if ui.button("Close").clicked() {
                                ui.close();
                            }
                        });
                    },
                );
            });
            if modal.should_close() {
                self.settings.open = false;
            }
        }
    }
}
