//! Bladvak

#![warn(clippy::all, rust_2018_idioms)]
#![deny(
    missing_docs,
    clippy::all,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::pedantic
)]
#![warn(clippy::multiple_crate_versions)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

pub mod app;
pub mod errors;
pub mod file_handler;
pub mod settings;
pub mod utils;

pub use app::{Bladvak, BladvakApp, MainResult};
pub use errors::{AppError, ErrorManager};
pub use file_handler::File;

/// re-export
pub use eframe;
pub use egui_extras;
pub use egui_plot;
pub use log;
pub use rfd;
pub use serde;

/// re-export wasm
#[cfg(target_arch = "wasm32")]
pub use js_sys;
#[cfg(target_arch = "wasm32")]
pub use web_sys;
