//! Bladvak

#![warn(clippy::all, rust_2018_idioms)]
#![deny(
    missing_docs,
    clippy::all,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cargo
)]
#![warn(clippy::multiple_crate_versions)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

pub mod app;
pub mod errors;
pub mod file_handler;
pub mod settings;
pub mod utils;

pub use app::{Bladvak, BladvakApp};
pub use errors::{AppError, ErrorManager};

/// eframe re-export
pub mod eframe {
    pub use eframe::*;
}

/// egui_extras re-export
pub mod egui_extras {
    pub use egui_extras::*;
}

/// log re-export
pub mod log {
    pub use log::*;
}

/// rfd re-export
pub mod rfd {
    pub use rfd::*;
}
