//! Documents

use std::slice::{Iter, IterMut};

/// Documents
#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub struct Documents<D> {
    /// current index
    pub(crate) current_idx: usize,
    /// documents
    inner: Vec<D>,
}

impl<D> Documents<D> {
    /// get current index
    #[must_use]
    pub fn get_current_index(&self) -> usize {
        self.current_idx
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
