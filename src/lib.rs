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

/// re-export
mod export {
    /// egui re-export
    mod egui {
        #[allow(unused_imports)]
        pub use egui::*;
    }
    /// eframe re-export
    mod eframe {
        #[allow(unused_imports)]
        pub use eframe::*;
    }
}
