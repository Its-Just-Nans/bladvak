//! Documents

use std::{
    fmt::{Debug, Display},
    path::{Path, PathBuf},
    slice::{Iter, IterMut},
};

use eframe::egui::{self, Id, Modal};

/// Document trait
pub trait DocumentTrait {
    /// show the name
    fn name(&self) -> impl Display {
        let filename = self
            .path()
            .file_name()
            .unwrap_or_else(|| self.path().as_os_str());
        filename.display()
    }

    /// Get the path of the document
    fn path(&self) -> &Path;

    /// Get the mut path of the document
    fn set_path(&mut self, new_path: PathBuf);
}

/// change name modal
#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub(crate) struct ChangeNameModal {
    /// current name
    current_name: String,
    /// index
    index: usize,
}

/// Documents
#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub struct Documents<D> {
    /// current index
    pub(crate) current_idx: usize,
    /// documents
    inner: Vec<D>,

    /// modal
    #[serde(skip)]
    pub(crate) change_name_modal: Option<ChangeNameModal>,
}

impl<D> Documents<D> {
    /// get current index
    #[must_use]
    pub fn get_current_index(&self) -> usize {
        self.current_idx
    }

    /// get document at index
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&D> {
        self.inner.get(index)
    }

    /// get document mut at index
    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut D> {
        self.inner.get_mut(index)
    }

    /// set the current index
    /// # Errors
    /// Return error if the index is incorrect
    pub fn set_current_index(&mut self, idx: usize) -> Result<(), String> {
        if idx >= self.inner.len() {
            return Err("Not enough documents".to_string());
        }
        Ok(())
    }

    /// get current document
    #[must_use]
    pub fn get_current_doc(&self) -> Option<&D> {
        if self.inner.is_empty() {
            return None;
        }
        let idx = self.current_idx % self.inner.len();
        Some(&self.inner[idx])
    }

    /// get current document as mutable
    pub fn get_current_doc_mut(&mut self) -> Option<&mut D> {
        if self.inner.is_empty() {
            return None;
        }
        let idx = self.current_idx % self.inner.len();
        Some(&mut self.inner[idx])
    }

    /// add a new document
    pub fn push(&mut self, document: D) {
        self.inner.push(document);
        self.current_idx = self.inner.len() - 1;
    }

    /// iter mut on documents
    pub fn iter_mut(&mut self) -> IterMut<'_, D> {
        self.inner.iter_mut()
    }

    /// iter on documents
    pub fn iter(&self) -> Iter<'_, D> {
        self.inner.iter()
    }

    /// Remove a document
    pub fn remove(&mut self, index: usize) {
        self.inner.remove(index);
        self.current_idx = self.current_idx.saturating_sub(1);
    }

    /// Check if is some
    #[must_use]
    pub fn is_some(&self) -> bool {
        !self.inner.is_empty()
    }

    /// Clear
    pub fn clear(&mut self) {
        self.current_idx = 0;
        self.inner.clear();
    }
}

impl<'a, D> IntoIterator for &'a Documents<D> {
    type Item = &'a D;
    type IntoIter = std::slice::Iter<'a, D>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D> IntoIterator for &'a mut Documents<D> {
    type Item = &'a mut D;
    type IntoIter = std::slice::IterMut<'a, D>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<D> Documents<D>
where
    D: DocumentTrait + Debug + Default,
{
    /// Show list of file
    pub fn show_file_list(&mut self, ui: &mut egui::Ui) {
        let mut current_idx = self.current_idx;
        let mut to_remove = None;

        for (idx, one_doc) in self.inner.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut current_idx, idx, format!("{}", one_doc.name()))
                    .context_menu(|ui| {
                        if ui.button("Change name").clicked() {
                            self.change_name_modal = Some(ChangeNameModal {
                                current_name: one_doc.path().to_string_lossy().to_string(),
                                index: current_idx,
                            });
                        }

                        if ui.button("Close").clicked() {
                            to_remove = Some(idx);
                        }
                    });
                if ui.button("x").clicked() {
                    to_remove = Some(idx);
                }
            });
            ui.separator();
        }
        self.current_idx = current_idx;
        if let Some(index) = to_remove {
            self.remove(index);
        }
        self.show_modal_name(ui);
    }

    /// show modal name edition
    fn show_modal_name(&mut self, ui: &mut egui::Ui) {
        let mut new_name = None;
        let mut should_close = false;
        if let Some(modal) = &mut self.change_name_modal {
            let modal = Modal::new(Id::new("Modal new image")).show(ui.ctx(), |ui| {
                ui.label("Create a new image");
                ui.horizontal(|ui| {
                    ui.label("Width");
                    ui.text_edit_singleline(&mut modal.current_name);
                });
                egui::Sides::new().show(
                    ui,
                    |modal_ui| {
                        if modal_ui.button("Cancel").clicked() {
                            should_close = true;
                            modal_ui.close();
                        }
                    },
                    |modal_ui| {
                        if modal_ui.button("Edit").clicked() {
                            new_name = Some((modal.index, modal.current_name.clone()));
                            modal_ui.close();
                        }
                    },
                );
            });
            if modal.should_close() {
                should_close = true;
            }
        }
        if let Some((idx, name)) = new_name {
            let Some(document) = self.get_mut(idx) else {
                return;
            };
            document.set_path(PathBuf::from(name));
            should_close = true;
        }
        if should_close {
            self.change_name_modal = None;
        }
    }
}
