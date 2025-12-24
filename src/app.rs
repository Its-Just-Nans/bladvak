//! App and app wrapper definitions

use eframe::{CreationContext, egui};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};

use crate::{
    errors::{AppError, ErrorManager},
    file_handler::{File, FileHandler},
    settings::Settings,
};

/// App trait
pub trait BladvakApp<'a>: Sized {
    /// Top panel ui
    fn top_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager);
    /// Setting panel ui
    fn panel_list(&self) -> Vec<Box<dyn BladvakPanel<App = Self>>>;
    /// Central panel ui
    fn central_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager);
    /// Side panel panel ui
    fn side_panel(&mut self, ui: &mut egui::Ui, func_ui: impl FnOnce(&mut egui::Ui, &mut Self));

    /// handle a file input
    /// # Errors
    /// Can return an error if fails to handle file
    fn handle_file(&mut self, bytes: File) -> Result<(), AppError>;
    /// hook on the file menu
    fn menu_file(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager);
    /// app name
    fn name() -> String;
    /// app version
    fn version() -> String;
    /// repo URL
    fn repo_url() -> String;
    /// icon
    fn icon() -> &'a [u8];

    /// should display a side_panel
    fn is_open_button(&self) -> bool;
    /// should display a side_panel
    fn is_side_panel(&self) -> bool;

    /// Builder func for native
    ///
    /// This functions is called as native AND in web - use [`crate::utils::is_native`] to make conditional code
    ///
    /// # Errors
    /// Can return an error if fails to create new app
    fn try_new_with_args(
        saved_state: Self,
        cc: &CreationContext<'_>,
        args: &[String],
    ) -> Result<Self, AppError>;
}

/// Trait for Bladvak panel
pub trait BladvakPanel: Debug {
    /// Type of the argument - the current App
    type App;

    /// Name of the panel
    fn name(&self) -> &str;

    /// Does this panel has a setting ui
    fn has_settings(&self) -> bool;

    /// Panel settings ui
    fn ui_settings(&self, app: &mut Self::App, ui: &mut egui::Ui, error_manager: &mut ErrorManager);

    /// Does this panel has an ui
    fn has_ui(&self) -> bool;

    /// Panel ui
    fn ui(&self, app: &mut Self::App, ui: &mut egui::Ui, error_manager: &mut ErrorManager);
}

/// Panel open state
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PanelOpen {
    #[default]
    /// In a window
    AsWindows,
    /// In sidebar
    AsSideBar,
    /// Hidden state
    None,
}

/// Panel state
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PanelState {
    /// open state of the panel
    pub(crate) open: PanelOpen,
}

/// Bladvak internal saved state
#[derive(Debug, Serialize, Deserialize)]
pub struct BladvakSavedState {
    /// settings
    pub(crate) settings: Settings,
    /// Panel state
    pub(crate) panel_state: BTreeMap<String, PanelState>,
}

/// App wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct Bladvak<App> {
    /// app
    pub(crate) app: App,

    /// Bladvak internal saved state
    pub(crate) internal: BladvakSavedState,

    /// error manager/handler
    #[serde(skip)]
    pub(crate) error_manager: ErrorManager,

    /// File Handler
    #[serde(skip)]
    pub(crate) file_handler: FileHandler,

    /// panel list
    #[serde(skip)]
    pub(crate) panel_list: Vec<Box<dyn BladvakPanel<App = App>>>,
}

/// Return type for bladvak_main
#[cfg(not(target_arch = "wasm32"))]
pub type MainResult = eframe::Result;

/// Return type for bladvak_main - wasm
#[cfg(target_arch = "wasm32")]
pub type MainResult = ();

impl<M> Bladvak<M>
where
    M: for<'a> BladvakApp<'a> + Debug + Default + Serialize + for<'a> Deserialize<'a> + 'static,
{
    /// Try to create a new app with args
    /// # Errors
    /// Can return an error if fails to create new app
    fn try_new_with_args(cc: &CreationContext<'_>, vec_args: &[String]) -> Result<Self, AppError> {
        let saved_state = if let Some(saved) = Self::get_saved_app_state(cc) {
            log::info!("Using saved state");
            (saved.app, Some(saved.internal))
        } else {
            (M::default(), None)
        };
        let app = M::try_new_with_args(saved_state.0, cc, vec_args)?;
        let panel_list = app.panel_list();
        let bladvak_internal = if let Some(saved_state) = saved_state.1
            && saved_state.panel_state.len() == panel_list.len()
        {
            // maybe add a check on the key of the panel_list
            log::info!("Using saved panels state");
            saved_state
        } else {
            let mut panel_state = BTreeMap::new();
            for one_panel in &panel_list {
                panel_state.insert(one_panel.name().to_string(), PanelState::default());
            }
            BladvakSavedState {
                settings: Default::default(),
                panel_state,
            }
        };
        Ok(Self {
            app,
            internal: bladvak_internal,
            error_manager: Default::default(),
            file_handler: Default::default(),
            panel_list,
        })
    }

    /// Show the central panel
    pub fn central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(&ctx.style())
                    .inner_margin(0)
                    .outer_margin(0),
            )
            .show(ctx, |ui| {
                self.app.central_panel(ui, &mut self.error_manager);
                for one_panel in self.panel_list.iter().filter(|p| p.has_ui()) {
                    let panel_name = one_panel.name();
                    if let Some(panel_state) = self.internal.panel_state.get_mut(panel_name)
                        && let PanelOpen::AsWindows = panel_state.open
                    {
                        let mut open = true;
                        egui::Window::new(panel_name)
                            .open(&mut open)
                            .show(ui.ctx(), |window_ui| {
                                one_panel.ui(&mut self.app, window_ui, &mut self.error_manager);
                            });
                        if !open {
                            panel_state.open = PanelOpen::AsSideBar;
                        }
                    }
                }
            });
    }

    /// Show the top panel
    pub fn top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    self.app.menu_file(ui, &mut self.error_manager);
                    if self.app.is_open_button() && ui.button("Open").clicked() {
                        ui.close();
                        self.file_handler.handle_file_open();
                    }
                    if ui.button("Settings").clicked() {
                        self.internal.settings.open = true;
                    }
                    let is_web = cfg!(target_arch = "wasm32");
                    if !is_web && ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    egui::warn_if_debug_build(ui);
                });
                self.app.top_panel(ui, &mut self.error_manager);
            });
        });
    }

    /// Show the side panel
    pub fn side_panel(&mut self, ctx: &egui::Context) {
        let is_panels_in_sidebar = self
            .internal
            .panel_state
            .iter()
            .any(|e| e.1.open == PanelOpen::AsSideBar);
        if is_panels_in_sidebar {
            egui::SidePanel::right("my_panel")
                .frame(
                    egui::Frame::central_panel(&ctx.style())
                        .inner_margin(0)
                        .outer_margin(0),
                )
                .min_width(self.internal.settings.min_width_sidebar)
                .show(ctx, |side_panel_ui| {
                    self.app.side_panel(side_panel_ui, |ui, app| {
                        for (idx, one_panel) in
                            self.panel_list
                                .iter()
                                .filter(|p| {
                                    p.has_ui()
                                        && self.internal.panel_state.get(p.name()).is_some_and(
                                            |p_state| p_state.open == PanelOpen::AsSideBar,
                                        )
                                })
                                .enumerate()
                        {
                            if idx != 0 {
                                ui.separator();
                            }
                            one_panel.ui(app, ui, &mut self.error_manager);
                        }
                        // self.app.side_panel(side_panel_ui, &mut self.error_manager);
                        ui.with_layout(
                            egui::Layout::bottom_up(egui::Align::RIGHT),
                            |ui: &mut egui::Ui| {
                                egui::warn_if_debug_build(ui);
                            },
                        );
                    });
                });
        }
    }

    /// When compiling natively
    /// # Errors
    /// Can return an error if fails to create new app
    #[cfg(not(target_arch = "wasm32"))]
    pub fn bladvak_main() -> eframe::Result {
        use std::env;

        use crate::app::Bladvak;

        env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

        let ico = match eframe::icon_data::from_png_bytes(M::icon()) {
            Ok(ico) => ico,
            Err(e) => {
                return Err(eframe::Error::AppCreation(
                    format!("Error loading the ico: {e}").into(),
                ));
            }
        };
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_drag_and_drop(true)
                .with_inner_size([400.0, 300.0])
                .with_min_inner_size([300.0, 220.0])
                .with_icon(ico),
            ..Default::default()
        };
        let args: Vec<String> = env::args().collect();

        eframe::run_native(
            &M::name(),
            native_options,
            Box::new(|cc| Ok(Box::new(Bladvak::<M>::try_new_with_args(cc, &args)?))),
        )
    }

    /// When compiling to web using trunk:
    #[cfg(target_arch = "wasm32")]
    pub fn bladvak_main() {
        use eframe::wasm_bindgen::JsCast as _;

        // Redirect `log` message to `console.log` and friends:
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();

        let web_options = eframe::WebOptions::default();

        wasm_bindgen_futures::spawn_local(async {
            let document = web_sys::window()
                .expect("No window")
                .document()
                .expect("No document");

            let canvas = document
                .get_element_by_id("the_canvas_id")
                .expect("Failed to find the_canvas_id")
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .expect("the_canvas_id was not a HtmlCanvasElement");

            let start_result = eframe::WebRunner::new()
                .start(
                    canvas,
                    web_options,
                    Box::new(|cc| {
                        match Bladvak::<M>::try_new_with_args(
                            cc,
                            &["sqfd".to_string(), "sqfd".to_string()],
                        ) {
                            Ok(app) => Ok(Box::new(app)),
                            Err(e) => {
                                log::error!("Failed to create app: {e}");
                                return Err(format!("Failed to create app: {e}").into());
                            }
                        }
                    }),
                )
                .await;

            // Remove the loading text and spinner:
            if let Some(loading_text) = document.get_element_by_id("loading_text") {
                match start_result {
                    Ok(_) => {
                        loading_text.remove();
                    }
                    Err(e) => {
                        loading_text.set_inner_html(
                            "<p> The app has crashed. See the developer console for details. </p>",
                        );
                        panic!("Failed to start app: {e:?}");
                    }
                }
            }
        });
    }

    /// Load previous app state (if any)
    // eframe: Note that you must enable the `persistence` feature for this to work.
    pub fn get_saved_app_state(cc: &eframe::CreationContext<'_>) -> Option<Bladvak<M>> {
        if let Some(storage) = cc.storage
            && let Some(saved_app_state) = eframe::get_value::<Bladvak<M>>(storage, eframe::APP_KEY)
        {
            log::info!("Loading saved app state");
            return Some(saved_app_state);
        }
        None
    }
}

impl<M> eframe::App for Bladvak<M>
where
    M: for<'a> BladvakApp<'a> + Debug + Default + Serialize + for<'a> Deserialize<'a> + 'static,
{
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.top_panel(ctx);

        if self.app.is_side_panel() {
            self.side_panel(ctx);
        }

        self.central_panel(ctx);

        match self.file_handler.handle_files(ctx) {
            Ok(Some(file)) => {
                if let Err(err) = self.app.handle_file(file) {
                    self.error_manager.add_error(err);
                }
                // repaint with the file
                ctx.request_repaint();
            }
            Ok(None) => {
                // nothing to do
            }
            Err(err) => {
                self.error_manager.add_error(err);
            }
        };

        self.show_error_manager(ctx);
        self.show_setting(ctx, frame);
    }
}
