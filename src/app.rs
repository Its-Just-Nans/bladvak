//! App and app wrapper definitions

use eframe::CreationContext;
use egui::ThemePreference;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::{
    errors::{AppError, ErrorManager},
    file_handler::FileHandler,
    settings::Settings,
};

/// App trait
pub trait BladvakApp: Sized {
    /// Top panel ui
    fn top_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager);
    /// Setting panel ui
    fn settings(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager);
    /// Central panel ui
    fn central_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager);
    /// Side panel ui
    fn side_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager);
    /// handle a file input
    /// # Errors
    /// Can return an error if fails to handle file
    fn handle_file(&mut self, bytes: &[u8]) -> Result<(), AppError>;
    /// hook on the file menu
    fn menu_file(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager);
    /// app name
    fn name() -> String;
    /// repo URL
    fn repo_url() -> String;

    /// should display a side_panel
    fn is_open_button(&self) -> bool;
    /// should display a side_panel
    fn is_side_panel(&self) -> bool;

    /// Builder func
    /// # Errors
    /// Can return an error if fails to create new app
    fn new(cc: &CreationContext<'_>) -> Result<Self, AppError>;
    /// Builder func for native
    /// # Errors
    /// Can return an error if fails to create new app
    #[cfg(not(target_arch = "wasm32"))]
    fn new_with_args(cc: &CreationContext<'_>, args: &[String]) -> Result<Self, AppError>;
}

/// App wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct Bladvak<App> {
    /// app
    pub(crate) app: App,
    /// settings
    pub(crate) settings: Settings,

    /// error manager/handler
    #[serde(skip)]
    pub(crate) error_manager: ErrorManager,

    /// File Handler
    pub(crate) file_handler: FileHandler,
}

impl<M> Bladvak<M>
where
    M: BladvakApp + Debug + Serialize + for<'a> Deserialize<'a> + 'static,
{
    /// Try to create a new app with args
    /// # Errors
    /// Can return an error if fails to create new app
    #[cfg(not(target_arch = "wasm32"))]
    pub fn try_new_with_args(
        cc: &CreationContext<'_>,
        vec_args: &[String],
    ) -> Result<Self, AppError> {
        let app = M::new_with_args(cc, vec_args)?;
        Ok(Self::new_with_app(app))
    }

    /// Try to create a new app
    /// # Errors
    /// Can return an error if fails to create new app
    pub fn try_new(cc: &CreationContext<'_>) -> Result<Self, AppError> {
        let app = M::new(cc)?;
        Ok(Self::new_with_app(app))
    }

    /// helper to create a new app
    pub fn new_with_app(app: M) -> Self {
        Self {
            app,
            settings: Default::default(),
            error_manager: Default::default(),
            file_handler: Default::default(),
        }
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
            });
    }

    /// Show the top panel
    pub fn top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    let is_web = cfg!(target_arch = "wasm32");
                    if !is_web && ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("Settings").clicked() {
                        self.settings.open = true;
                    }
                    if self.app.is_open_button() && ui.button("Open").clicked() {
                        ui.close();
                        self.file_handler.handle_file_open();
                    }
                    self.app.menu_file(ui, &mut self.error_manager);
                    ui.menu_button("Theme", |ui| {
                        let mut theme_preference = ui.ctx().options(|opt| opt.theme_preference);
                        ui.selectable_value(
                            &mut theme_preference,
                            ThemePreference::Light,
                            "☀ Light",
                        );
                        ui.selectable_value(
                            &mut theme_preference,
                            ThemePreference::Dark,
                            "🌙 Dark",
                        );
                        ui.selectable_value(
                            &mut theme_preference,
                            ThemePreference::System,
                            "💻 System",
                        );
                        ui.ctx().set_theme(theme_preference);
                    });
                    ui.add(
                        egui::Hyperlink::from_label_and_url("Repo", M::repo_url())
                            .open_in_new_tab(true),
                    );
                    egui::warn_if_debug_build(ui);
                });
                self.app.top_panel(ui, &mut self.error_manager);
            });
        });
    }

    /// Show the side panel
    pub fn side_panel(&mut self, ctx: &egui::Context) {
        if self.app.is_side_panel() {
            egui::SidePanel::right("my_panel")
                .frame(
                    egui::Frame::central_panel(&ctx.style())
                        .inner_margin(0)
                        .outer_margin(0),
                )
                .min_width(self.settings.min_width_sidebar)
                .show(ctx, |side_panel_ui| {
                    self.app.side_panel(side_panel_ui, &mut self.error_manager);
                    side_panel_ui.with_layout(
                        egui::Layout::bottom_up(egui::Align::LEFT),
                        |ui: &mut egui::Ui| {
                            egui::warn_if_debug_build(ui);
                        },
                    );
                });
        }
    }

    /// When compiling natively
    /// # Errors
    /// Can return an error if fails to create new app
    #[cfg(not(target_arch = "wasm32"))]
    pub fn bladvak_main(icon: &[u8]) -> eframe::Result {
        use std::env;

        use crate::app::Bladvak;

        env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

        let ico = match eframe::icon_data::from_png_bytes(icon) {
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
            Box::new(|cc| {
                if args.is_empty() {
                    Ok(Box::new(Bladvak::<M>::try_new(cc)?))
                } else {
                    Ok(Box::new(Bladvak::<M>::try_new_with_args(cc, &args)?))
                }
            }),
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
                    Box::new(|cc| match Bladvak::<M>::try_new(cc) {
                        Ok(app) => Ok(Box::new(app)),
                        Err(e) => {
                            log::error!("Failed to create app: {e}");
                            return Err(format!("Failed to create app: {e}").into());
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
}

impl<M> eframe::App for Bladvak<M>
where
    M: BladvakApp + Debug + Serialize + for<'a> Deserialize<'a> + 'static,
{
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.top_panel(ctx);

        if self.settings.right_panel {
            self.side_panel(ctx);
        }

        self.central_panel(ctx);

        match self.file_handler.handle_files(ctx) {
            Ok(Some(file)) => {
                if let Err(e) = self.app.handle_file(file.as_ref()) {
                    self.error_manager.add_error(e);
                }
                self.file_handler.reset();
            }
            Ok(None) => {}
            Err(err) => {
                self.error_manager.add_error(err);
            }
        }

        self.show_setting(ctx);
    }
}
