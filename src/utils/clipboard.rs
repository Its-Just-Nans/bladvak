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

/// Image type
pub type ClipBoardImage = (Vec<u8>, usize, usize);

/// Clipboard method to get files async
#[derive(Default)]
pub struct BladvakClipBoard {
    /// promise file
    #[cfg(target_arch = "wasm32")]
    pub(crate) promise_file: Option<poll_promise::Promise<Result<Vec<u8>, String>>>,
    /// promise text
    #[cfg(target_arch = "wasm32")]
    pub(crate) promise_text: Option<poll_promise::Promise<Result<String, String>>>,
    /// promise image
    #[cfg(target_arch = "wasm32")]
    pub(crate) promise_image: Option<poll_promise::Promise<Result<Vec<u8>, String>>>,
    /// Files
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) files: Option<Vec<LazyFile>>,
    /// Text
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) text: Option<String>,
    /// Image
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) image: Option<ClipBoardImage>,
}

impl std::fmt::Debug for BladvakClipBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(not(target_arch = "wasm32"))]
        {
            f.debug_struct("BladvakClipBoard")
                .field("files", &self.files)
                .field("text", &self.text)
                .finish_non_exhaustive()
        }
        #[cfg(target_arch = "wasm32")]
        {
            f.debug_struct("BladvakClipBoard").finish_non_exhaustive()
        }
    }
}

impl BladvakClipBoard {
    /// Get files if any - need to be called multiple times (on web)
    pub fn files(&mut self, ctx: &egui::Context) -> Option<Result<Vec<LazyFile>, String>> {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(prom) = &self.promise_file {
                // TODO await promise
                match prom.ready() {
                    Some(Ok(file_data)) => {
                        let mut data = Vec::with_capacity(1);
                        data.push(LazyFile {
                            data: file_data.clone(),
                        });
                        self.promise_file = None;
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
            None
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = ctx;
            if let Some(files) = self.files.take() {
                return Some(Ok(files));
            }
            None
        }
    }

    /// Get text if any - need to be called multiple times (on web)
    pub fn text(&mut self, ctx: &egui::Context) -> Option<Result<String, String>> {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(prom) = &self.promise_text {
                match prom.ready() {
                    Some(Ok(data)) => {
                        let text_data = data.clone();
                        self.promise_text = None;
                        return Some(Ok(text_data));
                    }
                    Some(Err(err)) => return Some(Err(err.to_string())),
                    None => {
                        ctx.request_repaint();
                        // not ready
                        return None;
                    }
                }
            }
            None
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = ctx;
            if let Some(txt) = self.text.take() {
                return Some(Ok(txt));
            }
            None
        }
    }

    /// Get image if any - need to be called multiple times (on web)
    pub fn image(&mut self, ctx: &egui::Context) -> Option<Result<ClipBoardImage, String>> {
        #[cfg(target_arch = "wasm32")]
        {
            use image::{GenericImageView, ImageReader};
            use std::io::Cursor;
            if let Some(prom) = &self.promise_image {
                match prom.ready() {
                    Some(Ok(bytes)) => {
                        let data = bytes.clone();
                        self.promise_image = None;
                        let img = ImageReader::new(Cursor::new(&data))
                            .with_guessed_format()
                            .ok()?
                            .decode()
                            .ok()?;

                        // Dimensions
                        let (width, height) = img.dimensions();

                        // Raw RGBA8 pixels
                        let raw = img.to_rgba8().into_raw();
                        return Some(Ok((raw, width as usize, height as usize)));
                    }
                    Some(Err(err)) => return Some(Err(err.to_string())),
                    None => {
                        ctx.request_repaint();
                        // not ready
                        return None;
                    }
                }
            }
            None
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = ctx;
            if let Some(img) = self.image.take() {
                return Some(Ok(img));
            }
            None
        }
    }

    /// Launch a get text from clipboard. You need to call `Self::text()` to get the text (if there
    /// are some)
    /// # Errors
    /// Error if accessing the clipboard
    pub fn launch_get_text(&mut self) -> Result<(), String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut clipboard =
                arboard::Clipboard::new().map_err(|e| format!("Cannot access clipboard: {e}"))?;
            let text = clipboard
                .get()
                .text()
                .map_err(|e| format!("Cannot access clipboard: {e}"))?;
            self.text = Some(text);
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.promise_text = Some(get_clipboard_text());
        }
        Ok(())
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
            self.promise_file = Some(get_clipboard_file());
        }
        Ok(())
    }

    /// Launch a get file from clipboard
    /// # Errors
    /// Fails in case we cannot get the image or the clipboard
    pub fn launch_get_image(&mut self) -> Result<(), String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use image::{GenericImageView, ImageReader};
            use std::fs::File;
            let mut arboard =
                arboard::Clipboard::new().map_err(|e| format!("Cannot access clipboard: {e}"))?;
            if let Ok(image) = arboard.get_image() {
                self.image = Some((image.bytes.into_owned(), image.width, image.height));
            } else if let Ok(files) = arboard.get().file_list()
                && let Some(f) = files.into_iter().nth(0)
            {
                use std::io::BufReader;

                let path = std::path::PathBuf::from(f.to_string_lossy().trim_end_matches('\r'));
                let file = File::open(&path)
                    .map_err(|e| format!("Cannot open {}: {e}", path.display()))?;
                let buf_read = BufReader::new(file);
                let img = ImageReader::new(buf_read)
                    .with_guessed_format()
                    .map_err(|e| format!("Cannot read image: {e}"))?
                    .decode()
                    .map_err(|e| format!("Cannot decode image: {e}"))?;

                // Dimensions
                let (width, height) = img.dimensions();

                // Raw RGBA8 pixels
                let raw = img.to_rgba8().into_raw();
                self.image = Some((raw, width as usize, height as usize));
            } else {
                return Err("Cannot get image from clipboard".to_string());
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.promise_image = Some(get_clipboard_image());
        }
        Ok(())
    }
}

/// Get the image from clipboard - not supported on web
/// # Errors
/// Error every time since clipboard is not supported on web
#[cfg(target_arch = "wasm32")]
pub fn get_clipboard_image() -> poll_promise::Promise<Result<Vec<u8>, String>> {
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
            log::info!("Available types for clipboard paste: {:?}", types);

            let correct_type_index = types.iter().position(|t| {
                t.as_string()
                    .map(|s| s.starts_with("image"))
                    .unwrap_or(false)
            });
            let Some(correct_type_index) = correct_type_index else {
                return Err("No image in clipboard".to_string());
            };
            let mime = types.get(correct_type_index as u32).as_string().unwrap();

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
        Err("No file found".to_string())
    })
}

#[cfg(target_arch = "wasm32")]
fn get_clipboard_text() -> poll_promise::Promise<Result<String, String>> {
    use wasm_bindgen_futures::JsFuture;
    poll_promise::Promise::spawn_local(async {
        let window = web_sys::window().ok_or("No window")?;

        let clipboard = window.navigator().clipboard();

        let text = JsFuture::from(clipboard.read_text())
            .await
            .map_err(|e| format!("{e:?}"))?
            .as_string()
            .ok_or("Clipboard does not contain text".to_string())?;
        Ok(text)
    })
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
            log::info!("Available types for clipboard paste: {:?}", types);

            let correct_type_index = types
                .iter()
                .position(|t| {
                    t.as_string()
                        .map(|s| s.starts_with("image"))
                        .unwrap_or(false)
                })
                .unwrap_or(0) as u32;
            let mime = types.get(correct_type_index).as_string().unwrap();

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
        Err("No file found".to_string())
    })
}
