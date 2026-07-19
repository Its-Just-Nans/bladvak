//! Clipboard

use eframe::egui;

/// a lazy file
#[derive(Debug)]
pub struct LazyFile {
    /// path of the file
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) path: std::path::PathBuf,
    /// data of the file
    #[cfg(target_arch = "wasm32")]
    pub(crate) data: Vec<u8>,
}

impl LazyFile {
    /// get the file data
    /// # Errors
    /// Fails if we cannot read file
    pub fn get_data(self) -> Result<Vec<u8>, String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::{fs::File, io::Read};
            let mut file = File::open(&self.path)
                .map_err(|e| format!("Cannot open {}: {e}", self.path.display()))?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|e| format!("Cannot read file: {e}"))?;
            Ok(buf)
        }
        #[cfg(target_arch = "wasm32")]
        {
            return Ok(self.data);
        }
    }
}

/// Clipboard method to get files async
#[derive(Default)]
pub struct BladvakClipBoard {
    /// promise
    #[cfg(target_arch = "wasm32")]
    pub(crate) promise: Option<poll_promise::Promise<Result<Vec<u8>, String>>>,
    /// Files
    pub(crate) files: Option<Vec<LazyFile>>,
}

impl std::fmt::Debug for BladvakClipBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BladvakClipBoard")
            .field("files", &self.files)
            .finish_non_exhaustive()
    }
}

impl BladvakClipBoard {
    /// Get files if any - need to be called multiple times (on web)
    pub fn files(&mut self, ctx: &egui::Context) -> Option<Result<Vec<LazyFile>, String>> {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(prom) = &self.promise {
                // TODO await promise
                match prom.ready() {
                    Some(Ok(file_data)) => {
                        let mut data = Vec::with_capacity(1);
                        data.push(LazyFile {
                            data: file_data.clone(),
                        });
                        self.promise = None;
                        return Some(Ok(data));
                    }
                    Some(Err(err)) => return Some(Err(err.to_string())),
                    None => {
                        ctx.request_repaint();
                        // not ready
                        return None;
                    }
                }
            }
        }
        let _ = ctx;
        if let Some(files) = self.files.take() {
            return Some(Ok(files));
        }
        None
    }
    /// Launch a get file from clipboard. You need to call `Self::files()` to get the file (if there
    /// are some)
    /// # Errors
    /// Error if accessing the clipboard
    pub fn launch_get_file(&mut self) -> Result<(), String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut clipboard =
                arboard::Clipboard::new().map_err(|e| format!("Cannot access clipboard: {e}"))?;

            let files = clipboard
                .get()
                .file_list()
                .map_err(|e| format!("Cannot access clipboard: {e}"))?;
            if !files.is_empty() {
                self.files = Some(
                    files
                        .into_iter()
                        .map(|f| {
                            let path = std::path::PathBuf::from(
                                f.to_string_lossy().trim_end_matches('\r'),
                            );
                            LazyFile { path }
                        })
                        .collect(),
                );
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.promise = Some(get_clipboard_file());
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
fn get_clipboard_file() -> poll_promise::Promise<Result<Vec<u8>, String>> {
    use js_sys::Array;
    use js_sys::Uint8Array;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::wasm_bindgen::JsCast;
    poll_promise::Promise::spawn_local(async {
        let window = web_sys::window().ok_or("No window")?;

        let clipboard = window.navigator().clipboard();

        let items = JsFuture::from(clipboard.read())
            .await
            .map_err(|e| format!("{e:?}"))?
            .dyn_into::<Array>()
            .map_err(|_| "Failed to cast clipboard items".to_string())?;

        if items.length() == 0 {
            return Err("Clipboard is empty".to_string());
        }

        for i in 0..items.length() {
            let item: web_sys::ClipboardItem = items.get(i).unchecked_into();

            let types = item.types();

            for j in 0..types.length() {
                let mime = types.get(j).as_string().unwrap();

                let blob = JsFuture::from(item.get_type(&mime))
                    .await
                    .map_err(|e| format!("{e:?}"))?
                    .dyn_into::<web_sys::Blob>()
                    .map_err(|_| "Failed to cast Blob".to_string())?;

                let buffer = JsFuture::from(blob.array_buffer())
                    .await
                    .map_err(|e| format!("{e:?}"))?;

                let bytes = Uint8Array::new(&buffer).to_vec();

                return Ok(bytes);
            }
        }

        Err("No file found".to_string())
    })
}
