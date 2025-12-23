//! Settings component

use eframe::egui::{self, Context, Id, Modal};
use serde::{Deserialize, Serialize};

use crate::app::{Bladvak, BladvakApp};

/// Selected Setting
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub(crate) enum SelectedSetting {
    /// Default info setting
    General,
    /// Custom setting
    String(String),
}

/// Settings object
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct Settings {
    /// Is setting modal open
    pub open: bool,

    /// Minimum width for the sidebar
    pub min_width_sidebar: f32,

    /// Right panel toggle
    pub right_panel: bool,

    /// Debug and inspection toggle
    pub show_inspection: bool,

    /// Selected Panel
    pub selected_setting: SelectedSetting,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_inspection: false,
            open: false,
            min_width_sidebar: 200.0,
            right_panel: true,
            selected_setting: SelectedSetting::General,
        }
    }
}

impl<M> Bladvak<M>
where
    M: for<'a> BladvakApp<'a> + Serialize + for<'a> Deserialize<'a> + 'static,
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
                egui::SidePanel::left("left_panel_setting")
                    .resizable(true)
                    .frame(
                        egui::Frame::central_panel(&ctx.style())
                            .inner_margin(0)
                            .outer_margin(5.0),
                    )
                    .show_inside(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("Settings");
                        });
                        ui.separator();
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.with_layout(
                                egui::Layout::top_down_justified(egui::Align::LEFT),
                                |ui| {
                                    ui.selectable_value(
                                        &mut self.settings.selected_setting,
                                        SelectedSetting::General,
                                        "General",
                                    );

                                    for one_setting in self.settings_list.clone() {
                                        ui.selectable_value(
                                            &mut self.settings.selected_setting,
                                            SelectedSetting::String(one_setting.clone()),
                                            one_setting,
                                        );
                                    }
                                },
                            );
                        });
                    });
                let value = self.settings.selected_setting.clone();
                egui::TopBottomPanel::bottom("bottom_settings").show_inside(ui, |ui| {
                    egui::Sides::new().spacing(20.0).show(
                        ui,
                        |modal_ui_left| {
                            modal_ui_left.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                ui.label(format!("{}@{}", M::name(), M::version()));
                            });
                        },
                        |modal_ui_right| {
                            if modal_ui_right.button("Close").clicked() {
                                modal_ui_right.close();
                            }
                        },
                    );
                });
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    self.show_setting_for(value, ui, frame);
                });
            });
            if modal.should_close() {
                self.settings.open = false;
            }
        }
    }

    /// Show setting of selected
    pub(crate) fn show_setting_for(
        &mut self,
        selected: SelectedSetting,
        ui: &mut egui::Ui,
        frame: &mut eframe::Frame,
    ) {
        match selected {
            SelectedSetting::General => {
                ui.heading(format!("{} settings", M::name()));
                ui.horizontal(|ui| {
                    ui.label(format!("Reset storage of {}", M::name()));
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
                    ui.button("⟳")
                        .on_hover_text(format!("Reset {}", self.error_manager.title()))
                        .clicked()
                        .then(|| {
                            self.error_manager = Default::default();
                        });
                });
                ui.checkbox(&mut self.error_manager.is_open, "Show Error panel");
                ui.separator();
                ui.checkbox(&mut self.settings.show_inspection, "Show Debug panel");
                ui.separator();
                ui.heading("About");
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("Version: ");
                    ui.label(M::version());
                });
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("Link to ");
                    ui.add(
                        egui::Hyperlink::from_label_and_url(
                            format!("{} repository", M::name()),
                            M::repo_url(),
                        )
                        .open_in_new_tab(true),
                    );
                });
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("Using ");
                    ui.add(
                        egui::Hyperlink::from_label_and_url(
                            concat!("bladvak@", env!("CARGO_PKG_VERSION")),
                            "https://github.com/Its-Just-Nans/bladvak",
                        )
                        .open_in_new_tab(true),
                    );
                });
            }
            SelectedSetting::String(setting_name) => {
                ui.heading(format!("{setting_name} settings"));
                self.app
                    .show_setting_for(&setting_name, ui, &mut self.error_manager)
            }
        }
    }
}
