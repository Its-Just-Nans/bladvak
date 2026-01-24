//! utility functions

use std::path::Path;
use std::path::PathBuf;

use crate::AppError;

pub mod grid;

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
    let blob = web_sys::Blob::new_with_u8_array_sequence(&array_data)
        .map_err(|_| "Cannot create file data")?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|_| "Cannot create file url data")?;
    // create link
    let document = web_sys::window()
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
    a.dyn_ref::<web_sys::HtmlElement>()
        .ok_or("Cannot simulate click")?
        .click();
    // revoke url
    web_sys::Url::revoke_object_url(&url)
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
