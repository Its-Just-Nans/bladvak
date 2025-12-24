//! Settings component

use eframe::egui::{self, Context, Id, Modal, ThemePreference};
use serde::{Deserialize, Serialize};

use crate::app::{Bladvak, BladvakApp, PanelOpen};

/// Selected Setting
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub(crate) enum SelectedSetting {
    /// General setting
    General,
    /// Panel setting
    Panel,
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
            .open(&mut self.internal.settings.show_inspection)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.inspection_ui(ui);
            });
        if self.internal.settings.open {
            let modal = Modal::new(Id::new("Modal settings")).show(ctx, |ui| {
                let value = self.internal.settings.selected_setting.clone();
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
                                        &mut self.internal.settings.selected_setting,
                                        SelectedSetting::General,
                                        "General",
                                    );
                                    ui.selectable_value(
                                        &mut self.internal.settings.selected_setting,
                                        SelectedSetting::Panel,
                                        "Panels",
                                    );

                                    for one_panel in &self.panel_list {
                                        if one_panel.has_settings(&self.app) {
                                            let one_setting_name =
                                                one_panel.name(&self.app).to_string();
                                            ui.selectable_value(
                                                &mut self.internal.settings.selected_setting,
                                                SelectedSetting::String(one_setting_name.clone()),
                                                one_setting_name,
                                            );
                                        }
                                    }
                                    // }
                                },
                            );
                        });
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| match value {
                    SelectedSetting::General => {
                        self.show_general_setting(ui, frame);
                    }
                    SelectedSetting::Panel => {
                        self.show_panel_setting(ui);
                    }
                    SelectedSetting::String(value) => {
                        for one_panel in &self.panel_list {
                            let panel_name = one_panel.name(&self.app);
                            if panel_name == value {
                                ui.heading(format!("{} settings", panel_name));
                                ui.separator();
                                one_panel.ui_settings(&mut self.app, ui, &mut self.error_manager);
                            }
                        }
                    }
                });
            });
            if modal.should_close() {
                self.internal.settings.open = false;
            }
        }
    }

    /// Show setting of selected
    pub(crate) fn show_panel_setting(&mut self, ui: &mut egui::Ui) {
        ui.heading("Panels");
        for one_panel in &self.panel_list {
            if one_panel.has_ui(&self.app) {
                let panel_name = one_panel.name(&self.app).to_string();
                if let Some(state) = self.internal.panel_state.get_mut(&panel_name) {
                    let is_side_panel = self.app.is_side_panel();
                    ui.horizontal(|ui| {
                        ui.label(panel_name);
                        if is_side_panel {
                            ui.selectable_value(&mut state.open, PanelOpen::AsSideBar, "Sidebar");
                        } else if state.open == PanelOpen::AsSideBar {
                            // set the default to None (hidden)
                            state.open = PanelOpen::None
                        }
                        ui.selectable_value(&mut state.open, PanelOpen::AsWindows, "Windows");
                        ui.selectable_value(&mut state.open, PanelOpen::None, "None");
                    });
                }
            }
        }
    }

    /// Show setting of selected
    pub(crate) fn show_general_setting(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.heading(format!("{} settings", M::name()));
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(format!("Reset storage of {}", M::name()));
            ui.button("⟳").clicked().then(|| {
                if let Some(storage) = frame.storage_mut() {
                    eframe::set_value(storage, eframe::APP_KEY, self);
                    log::info!("Storage reset");
                }
            });
        });
        ui.horizontal(|ui| {
            ui.label(format!("Reset {}", self.error_manager.title()));
            ui.button("⟳").clicked().then(|| {
                self.error_manager = Default::default();
            });
        });
        ui.checkbox(&mut self.error_manager.is_open, "Show Error panel");
        ui.checkbox(
            &mut self.internal.settings.show_inspection,
            "Show Debug panel",
        );
        ui.separator();
        ui.heading("Theme");
        ui.horizontal(|ui| {
            let mut theme_preference = ui.ctx().options(|opt| opt.theme_preference);
            ui.selectable_value(&mut theme_preference, ThemePreference::Light, "☀ Light");
            ui.selectable_value(&mut theme_preference, ThemePreference::Dark, "🌙 Dark");
            ui.selectable_value(&mut theme_preference, ThemePreference::System, "💻 System");
            ui.ctx().set_theme(theme_preference);
        });
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
}
