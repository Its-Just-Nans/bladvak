//! File handler

use poll_promise::Promise;
use std::{fmt::Debug, fs::read, sync::Arc};

use crate::errors::AppError;

/// File object
#[derive(Default)]
pub struct File {
    /// File data
    pub data: Arc<Vec<u8>>,
}

/// File Handler
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct FileHandler {
    /// Dropped_files handler
    #[serde(skip)]
    pub dropped_files: Vec<egui::DroppedFile>,

    /// File upload handling
    #[serde(skip)]
    pub file_upload: Option<Promise<Result<File, AppError>>>,
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
            let file_selected = rfd::AsyncFileDialog::new().pick_file().await;
            if let Some(curr_file) = file_selected {
                let buf = curr_file.read().await;
                return Ok(File {
                    data: Arc::new(buf),
                });
            }
            // no file selected
            Err(AppError::new_fake("Upload: no file Selected".to_string()))
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
                    return Ok(File {
                        data: Arc::new(buf),
                    });
                }
            }
            // no file selected
            Err(AppError::new_fake("Upload: no file Selected".to_string()))
        }))
    }

    /// Reset the file_handler
    pub fn reset(&mut self) {
        self.file_upload = None;
    }

    /// Handle file upload
    fn handle_file_upload(&mut self) -> Result<Option<Arc<Vec<u8>>>, AppError> {
        match &self.file_upload {
            Some(result) => match result.ready() {
                Some(Ok(File { data, .. })) => {
                    let res = data;
                    Ok(Some(res.clone()))
                }
                Some(Err(e)) => {
                    let err = e.clone();
                    Err(err)
                }
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    /// Handle file dropped
    fn handle_file_dropped(&mut self) -> Result<Option<Arc<Vec<u8>>>, AppError> {
        if self.dropped_files.is_empty() {
            return Ok(None);
        }
        let file = self.dropped_files.remove(0);
        if cfg!(not(target_arch = "wasm32")) {
            if let Some(path) = file.path.as_deref() {
                let file = read(path)?;
                return Ok(Some(Arc::new(file)));
            }
        } else if cfg!(target_arch = "wasm32")
            && let Some(bytes) = file.bytes.as_deref()
        {
            return Ok(Some(Arc::new(bytes.to_vec())));
        }
        Ok(None)
    }

    /// Handle the files
    /// # Errors
    /// Can return an error if fails to handle files
    pub fn handle_files(&mut self, ctx: &egui::Context) -> Result<Option<Arc<Vec<u8>>>, AppError> {
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                // read the first file
                self.dropped_files.clone_from(&i.raw.dropped_files);
            }
        });
        if let Some(content) = self.handle_file_upload()? {
            return Ok(Some(content));
        }
        if let Some(content) = self.handle_file_dropped()? {
            return Ok(Some(content));
        }
        Ok(None)
    }
}
