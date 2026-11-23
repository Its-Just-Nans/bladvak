//! File handler

use eframe::egui::{self, Id, Modal};
use poll_promise::Promise;
use std::{fmt::Debug, path::PathBuf, sync::Arc};

use crate::errors::AppError;

/// File object
#[derive(Default, Clone)]
pub struct File {
    /// File data
    pub data: Vec<u8>,
    /// Path or filename
    pub path: PathBuf,
}

impl Debug for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_fmt = f.debug_struct("File");
        debug_fmt.field("path", &self.path);
        debug_fmt.field("data", &self.data.len());
        debug_fmt.finish()
    }
}

/// File Handler
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct FileHandler {
    /// Dropped files handler
    #[serde(skip)]
    pub dropped_files: Vec<Arc<dyn egui::DroppedFile + Send + Sync>>,

    /// File upload handling
    #[serde(skip)]
    pub file_upload: Option<Promise<Result<FileState, AppError>>>,

    /// From URL settings
    from_url: Option<(String, Method, String)>,
}

/// Method
#[derive(Debug, PartialEq, Copy, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) enum Method {
    /// GET
    Get,
    /// HEAD
    Head,
    /// POST
    Post,
}

/// File state
#[derive(Clone, Debug)]
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
            let val = String::new();
            debug_fmt.field("file_upload", &val);
        }
        debug_fmt.field("from_url", &self.from_url);
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
                }
                return Err(AppError::new("Invalid file path".to_string()));
            }
            // no file selected
            Ok(FileState::NotSelected)
        }));
    }

    /// Reset the `file_handler`
    pub fn reset(&mut self) {
        self.file_upload = None;
    }

    /// Handle file dropped
    fn handle_file_dropped(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            for one_file in &i.raw.dropped_files {
                self.dropped_files.push(one_file.clone());
            }
        });
        if self.dropped_files.is_empty() {
            return;
        }
        let file = self.dropped_files.remove(0);
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.file_upload = Some(Promise::spawn_thread("slow", move || {
                let path = file.path();
                let buf = std::fs::read(path);
                let buf = match buf {
                    Ok(v) => v,
                    Err(e) => {
                        log::warn!("{e:?}");
                        return Err(AppError::new(e.to_string()));
                    }
                };
                Ok(FileState::Ready(File {
                    data: buf,
                    path: path.to_path_buf(),
                }))
            }));
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.file_upload = Some(Promise::spawn_local(async move {
                let path = file.path();
                if let Ok(buf) = file.bytes_async().await {
                    return Ok(FileState::Ready(File {
                        data: buf,
                        path: path.to_path_buf(),
                    }));
                }
                // no file selected
                Ok(FileState::NotSelected)
            }));
        }
        // if cfg!(not(target_arch = "wasm32")) {
        //     let path = file.path();
        //     let file = read(path)?;
        //     return Ok(Some(File {
        //         data: file,
        //         path: path.to_path_buf(),
        //     }));
        // } else if cfg!(target_arch = "wasm32")
        //     && let Some(bytes) = file.bytes.as_deref()
        // {
        //     return Ok(Some(File {
        //         data: bytes.to_vec(),
        //         path: file.path.unwrap_or(PathBuf::from(file.name)),
        //     }));
        // }
    }

    /// Handle the files
    /// # Errors
    /// Can return an error if fails to handle files
    pub fn handle_files(&mut self, ctx: &egui::Context) -> Result<Option<File>, AppError> {
        let file_upload_state = match &self.file_upload {
            Some(result) => match result.ready() {
                Some(Ok(state)) => Ok(state.clone()),
                Some(Err(e)) => Err(e.clone()),
                None => Ok(FileState::UploadedOrSelected), // promise not ready
            },
            None => Ok(FileState::NoUpload), // no file upload
        };
        match file_upload_state {
            Ok(state) => match state {
                FileState::NotSelected => {
                    log::info!("No file selected");
                    self.reset();
                }
                FileState::UploadedOrSelected => {
                    log::info!("File is being uploaded or selected...");
                    ctx.request_repaint();
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
        self.handle_file_dropped(ctx);
        Ok(None)
    }

    /// Show ui of file handler
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let mut should_load = false;
        if let Some((url, method, body)) = &mut self.from_url {
            let modal = Modal::new(Id::unique("modal_file_handler_url")).show(ui.ctx(), |ui| {
                ui.label("Load from URL");
                ui.horizontal(|ui| {
                    ui.selectable_value(method, Method::Get, "Get");
                    ui.selectable_value(method, Method::Head, "Head");
                    ui.selectable_value(method, Method::Post, "Post");
                });
                ui.horizontal(|ui| {
                    ui.label("URL: ");
                    ui.text_edit_singleline(url);
                });
                if *method == Method::Post {
                    ui.text_edit_multiline(body);
                }
                egui::Sides::new().show(
                    ui,
                    |modal_ui| {
                        if modal_ui.button("Cancel").clicked() {
                            modal_ui.close();
                        }
                    },
                    |modal_ui| {
                        if modal_ui.button("Load").clicked() {
                            should_load = true;
                            modal_ui.close();
                        }
                    },
                );
            });
            if should_load {
                self.load_from_url();
            }

            if modal.should_close() {
                self.from_url = None;
            }
        }
    }

    /// open modal for URL
    pub fn open_file_from_url(&mut self) {
        self.from_url = Some((
            String::new(),
            Method::Get,
            r#"["posting some json"]"#.to_string(),
        ));
    }

    /// Load from URL
    fn load_from_url(&mut self) {
        if let Some((url, method, body)) = &self.from_url {
            log::info!("Loading from URL: {url}");
            let request = match method {
                Method::Get => ehttp::Request::get(url),
                Method::Head => ehttp::Request::head(url),
                Method::Post => ehttp::Request::post(url, body.as_bytes().to_vec()),
            };
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.file_upload = Some(Promise::spawn_thread("file_upload_from_url", move || {
                    let response = ehttp::fetch_blocking(&request);
                    match response {
                        Ok(body) => Ok(FileState::Ready(File {
                            data: body.bytes,
                            path: PathBuf::from("from_url"),
                        })),
                        Err(err) => Err(err.into()),
                    }
                }));
            }
            #[cfg(target_arch = "wasm32")]
            {
                self.file_upload = Some(Promise::spawn_local(async {
                    let resp_promise = ehttp::fetch_async(request);
                    let response = resp_promise.await;
                    match response {
                        Ok(body) => Ok(FileState::Ready(File {
                            data: body.bytes,
                            path: PathBuf::from("from_url"),
                        })),
                        Err(err) => Err(err.into()),
                    }
                }));
            }
        }
        self.from_url = None;
    }
}
