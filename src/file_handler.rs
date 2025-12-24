//! File handler

use eframe::egui;
use poll_promise::Promise;
use std::{fmt::Debug, fs::read, path::PathBuf};

use crate::errors::AppError;

/// File object
#[derive(Default, Clone)]
pub struct File {
    /// File data
    pub data: Vec<u8>,
    /// Path or filename
    pub path: PathBuf,
}

/// File Handler
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct FileHandler {
    /// Dropped_files handler
    #[serde(skip)]
    pub dropped_files: Vec<egui::DroppedFile>,

    /// File upload handling
    #[serde(skip)]
    pub file_upload: Option<Promise<Result<FileState, AppError>>>,
}

/// File state
#[derive(Clone)]
pub enum FileState {
    /// File is not selected
    NotSelected,
    /// File is being uploaded or selected
    UploadedOrSelected,
    /// No file upload
    NoUpload,
    /// File is ready
    Ready(File),
}

impl Debug for FileHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_fmt = f.debug_struct("FileHandler");
        debug_fmt.field("dropped_files", &self.dropped_files);
        if self.file_upload.is_some() {
            let val = "".to_string();
            debug_fmt.field("file_upload", &val);
        }
        debug_fmt.finish()
    }
}

impl FileHandler {
    /// Handle the file
    #[cfg(target_arch = "wasm32")]
    pub fn handle_file_open(&mut self) {
        self.file_upload = Some(Promise::spawn_local(async {
            log::info!("rfd start");
            let file_selected = rfd::AsyncFileDialog::new().pick_file().await;
            log::info!("rfd result {:?}", file_selected);
            if let Some(curr_file) = file_selected {
                let buf = curr_file.read().await;
                return Ok(FileState::Ready(File {
                    data: buf,
                    path: PathBuf::from(curr_file.file_name()),
                }));
            }
            // no file selected
            Ok(FileState::NotSelected)
        }));
    }

    /// Handle the file
    #[cfg(not(target_arch = "wasm32"))]
    pub fn handle_file_open(&mut self) {
        self.file_upload = Some(Promise::spawn_thread("slow", move || {
            if let Some(path_buf) = rfd::FileDialog::new().pick_file() {
                // read file as string
                if let Some(path) = path_buf.to_str() {
                    let buf = std::fs::read(path);
                    let buf = match buf {
                        Ok(v) => v,
                        Err(e) => {
                            log::warn!("{e:?}");
                            return Err(AppError::new(e.to_string()));
                        }
                    };
                    return Ok(FileState::Ready(File {
                        data: buf,
                        path: path_buf,
                    }));
                } else {
                    return Err(AppError::new("Invalid file path".to_string()));
                }
            }
            // no file selected
            Ok(FileState::NotSelected)
        }))
    }

    /// Reset the file_handler
    pub fn reset(&mut self) {
        self.file_upload = None;
    }

    /// Handle file upload
    fn handle_file_upload(&mut self) -> Result<FileState, AppError> {
        match &self.file_upload {
            Some(result) => match result.ready() {
                Some(Ok(state)) => Ok(state.clone()),
                Some(Err(e)) => Err(e.clone()),
                None => Ok(FileState::UploadedOrSelected), // promise not ready
            },
            None => Ok(FileState::NoUpload), // no file upload
        }
    }

    /// Handle file dropped
    fn handle_file_dropped(&mut self) -> Result<Option<File>, AppError> {
        if self.dropped_files.is_empty() {
            return Ok(None);
        }
        let file = self.dropped_files.remove(0);
        if cfg!(not(target_arch = "wasm32")) {
            if let Some(path) = file.path.as_deref() {
                let file = read(path)?;
                return Ok(Some(File {
                    data: file,
                    path: path.to_path_buf(),
                }));
            }
        } else if cfg!(target_arch = "wasm32")
            && let Some(bytes) = file.bytes.as_deref()
        {
            return Ok(Some(File {
                data: bytes.to_vec(),
                path: file.path.unwrap_or(PathBuf::from(file.name)),
            }));
        }
        Ok(None)
    }

    /// Handle the files
    /// # Errors
    /// Can return an error if fails to handle files
    pub fn handle_files(&mut self, ctx: &egui::Context) -> Result<Option<File>, AppError> {
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                // read the first file
                self.dropped_files.clone_from(&i.raw.dropped_files);
            }
        });
        match self.handle_file_upload() {
            Ok(state) => match state {
                FileState::NotSelected => {
                    log::info!("No file selected");
                    self.reset();
                }
                FileState::UploadedOrSelected => {
                    log::info!("File is being uploaded or selected...");
                    return Ok(None);
                }
                FileState::Ready(data) => {
                    log::info!("File uploaded successfully");
                    self.reset();
                    return Ok(Some(data));
                }
                FileState::NoUpload => {
                    self.reset();
                }
            },
            Err(e) => {
                self.reset();
                return Err(e);
            }
        }
        if let Some(file_dropped) = self.handle_file_dropped()? {
            return Ok(Some(file_dropped));
        }
        Ok(None)
    }
}
