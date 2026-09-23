//! utility functions

use eframe::egui;
use std::path::Path;
use std::path::PathBuf;

use crate::AppError;
use crate::ErrorManager;

pub mod clipboard;
pub mod document;
pub mod grid;

pub use clipboard::{BladvakClipBoard, LazyFile};
pub use document::Documents;

/// Save the data to a file
/// # Errors
/// Error if fails to save the file
#[cfg(not(target_arch = "wasm32"))]
pub fn save_file(data: &[u8], path_file: &Path) -> Result<(), String> {
    use std::fs::File;
    use std::io::prelude::*;

    let mut file = File::create(path_file).map_err(|e| format!("Cannot create file: {e}"))?;
    file.write_all(data)
        .map_err(|e| format!("Cannot write file: {e}"))
}

/// Save the data as file
/// # Errors
/// Error if fails to save the file
#[cfg(target_arch = "wasm32")]
pub fn save_file(data: &[u8], path_file: &Path) -> Result<(), String> {
    // create blob
    use eframe::wasm_bindgen::JsCast;
    use js_sys::Array;

    log::info!("Saving file to {:?}", path_file);
    let filename = match path_file.file_name() {
        Some(name) => name.to_str().ok_or("Cannot get filename")?,
        None => "file.png",
    };

    let array_data = Array::new();
    array_data.push(&js_sys::Uint8Array::from(data));
    let blob = eframe::web_sys::Blob::new_with_u8_array_sequence(&array_data)
        .map_err(|_| "Cannot create file data")?;
    let url = eframe::web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|_| "Cannot create file url data")?;
    // create link
    let document = eframe::web_sys::window()
        .ok_or("Cannot get the website window")?
        .document()
        .ok_or("Cannot get the website document")?;
    let a = document
        .create_element("a")
        .map_err(|_| "Cannot create <a> element")?;
    a.set_attribute("href", &url)
        .map_err(|_| "Cannot create add href attribute")?;
    a.set_attribute("download", filename)
        .map_err(|_| "Cannot create add download attribute")?;

    // click link
    a.dyn_ref::<eframe::web_sys::HtmlElement>()
        .ok_or("Cannot simulate click")?
        .click();
    // revoke url
    eframe::web_sys::Url::revoke_object_url(&url)
        .map_err(|_| "Cannot remove object url with revoke_object_url".into())
}

/// Get the save path
/// # Errors
/// Failed if the input is wrong
#[cfg(not(target_arch = "wasm32"))]
pub fn get_save_path(current_path: Option<&Path>) -> Result<Option<PathBuf>, AppError> {
    use rfd::FileDialog;
    let path = FileDialog::new()
        .set_directory(match &current_path {
            Some(path) => path.parent().ok_or("Cannot get parent in the path")?,
            None => std::path::Path::new("."),
        })
        .set_file_name(match &current_path {
            Some(path) => path
                .file_name()
                .ok_or("Cannot get file name")?
                .to_string_lossy(),
            None => std::path::Path::new("file").to_string_lossy(),
        })
        .save_file();
    Ok(path)
}
/// Get a new path
/// # Errors
/// No error in wasm
#[cfg(target_arch = "wasm32")]
pub fn get_save_path(current_path: Option<&Path>) -> Result<Option<PathBuf>, AppError> {
    match current_path {
        Some(p) => Ok(Some(p.to_path_buf())),
        None => Ok(Some(PathBuf::from("file"))),
    }
}

/// Is running on web
#[inline]
#[must_use]
pub const fn is_web() -> bool {
    cfg!(target_arch = "wasm32")
}

/// Is running on native
#[inline]
#[must_use]
pub const fn is_native() -> bool {
    !is_web()
}

/// Copy the image to clipboard
/// # Errors
/// Error if fails to copy the image to clipboard
pub fn set_image_in_clipboard(
    ctx: &eframe::egui::Context,
    width: usize,
    height: usize,
    data: &[u8],
) -> Result<(), String> {
    use eframe::egui::ColorImage;
    let size = [width as _, height as _];
    let color_image = ColorImage::from_rgba_unmultiplied(size, data);
    ctx.copy_image(color_image);
    Ok(())
}

/// Central ui
pub fn central_ui(ui: &mut egui::Ui, inner_ui: impl FnOnce(&mut egui::Ui)) {
    egui::Area::new("center".into())
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                inner_ui(ui);
            });
        });
}

/// Show a size
pub fn show_size(ui: &mut egui::Ui, size: usize) {
    #[allow(clippy::cast_precision_loss)]
    let size = size as f64;
    let default_size = if size > (1000.0 * 1000.0) {
        format!("Size: {:.3} MB", size / 1000.0 / 1000.0)
    } else if size > 1000.0 {
        format!("Size: {:.3} kB", size / 1000.0)
    } else {
        format!("Size: {size} bytes")
    };
    egui::CollapsingHeader::new(default_size)
        .id_salt(format!("show_size_{size}"))
        .show(ui, |ui| {
            ui.label(format!("{size} bytes"));
            egui::Grid::new(format!("show_size_table_{size}"))
                .striped(true)
                .show(ui, |ui| {
                    ui.label(format!("{:.3} kB", size / 1000.0))
                        .on_hover_text("1000");
                    ui.label(format!("{:.3} KiB", size / 1024.0))
                        .on_hover_text("1024");
                    ui.end_row();
                    ui.label(format!("{:.3} MB", size / 1000.0 / 1000.0))
                        .on_hover_text("1000^2");
                    ui.label(format!("{:.3} MiB", size / 1024.0 / 1024.0))
                        .on_hover_text("1024^2");
                    ui.end_row();
                });
        });
}

/// custom collapsing
pub fn custom_collapsing_header(
    ui: &mut egui::Ui,
    id: impl Into<egui::Id>,
    ui_header: impl FnOnce(&mut egui::Ui),
    ui_body: impl FnOnce(&mut egui::Ui),
) {
    let id = ui.make_persistent_id(id.into());
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
        .show_header(ui, |ui| {
            ui_header(ui);
        })
        .body(|ui| {
            ui_body(ui);
        });
}

/// open external
#[cfg(target_arch = "wasm32")]
pub fn open_external(
    data: Vec<u8>,
    filename: &Path,
    error_manager: &mut ErrorManager,
    target: &str,
) {
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    use {js_sys, web_sys};
    let target = target.to_string();

    let Some(window) = web_sys::window() else {
        error_manager.add_error("Cannot create window");
        return;
    };

    let parts = js_sys::Array::new();
    let bytes = js_sys::Uint8Array::from(data.as_slice());
    parts.push(&bytes);

    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/octet-stream");

    let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&parts, &options) else {
        error_manager.add_error("Cannot create blob");
        return;
    };

    // Open Tab B
    let Ok(res_tab) = window.open_with_url_and_target(&target, "_blank") else {
        error_manager.add_error("Cannot open URL");
        return;
    };

    let Some(tab_b) = res_tab else {
        error_manager.add_error("Cannot get WindowProxy");
        return;
    };

    let message = js_sys::Object::new();

    if let Err(_err) = js_sys::Reflect::set(&message, &JsValue::from_str("blob"), &blob) {
        error_manager.add_error("Cannot had blob to message");
        return;
    };
    if let Err(_err) = js_sys::Reflect::set(
        &message,
        &JsValue::from_str("filename"),
        &JsValue::from_str(&filename.to_string_lossy().to_string()),
    ) {
        error_manager.add_error("Cannot had filename to message");
        return;
    };
    let callback = Closure::once(move || {
        // Send to Tab B
        if let Err(_err) = tab_b.post_message(&message, &target) {
            return;
        }
    });

    if let Err(_err) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        callback.as_ref().unchecked_ref(),
        1500,
    ) {
        error_manager.add_error("Cannot set timeout");
        return;
    };

    callback.forget();
}

/// open external
#[cfg(not(target_arch = "wasm32"))]
pub fn open_external(
    _data: Vec<u8>,
    _filename: &Path,
    _error_manager: &mut ErrorManager,
    _target: &str,
) {
    // TODO:
}
