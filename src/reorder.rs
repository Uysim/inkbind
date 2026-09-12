//! Reordering the page sequence of a PDF document.
//!
//! Implemented as of Step 11. [`Document::reorder`] clones the source
//! document (the same technique [`crate::split`] uses) and rewrites only
//! the root `/Pages` dictionary's `/Kids` array to the given order —
//! `/Count` is unchanged (the page set is the same, only its order moves),
//! and every page's `/Parent` chain is left untouched, same precedent as
//! [`crate::split`]. `lopdf` is an implementation detail: its types never
//! appear in this module's public API.

use lopdf::{Object, ObjectId};

use crate::document::{map_lopdf_error, Document};
use crate::{Error, Result};

impl Document {
    /// Reorders the document's pages according to `order`: the output's
    /// page at position `i` is this document's page at index `order[i]`.
    ///
    /// `order` must be a permutation of `0..self.page_count()` — the same
    /// length, with every index in range and no duplicates. Returns
    /// [`Error::PageNotFound`] if an entry is out of range, or
    /// [`Error::InvalidPageOrder`] if the length doesn't match or an index
    /// repeats.
    ///
    /// ```
    /// let doc = inkbind::Document::open("tests/fixtures/two_pages.pdf")?;
    ///
    /// let reversed = doc.reorder(&[1, 0])?;
    /// assert!(reversed.page(0)?.text()?.contains("Page Two"));
    /// assert!(reversed.page(1)?.text()?.contains("Page One"));
    /// # Ok::<(), inkbind::Error>(())
    /// ```
    pub fn reorder(&self, order: &[usize]) -> Result<Document> {
        let page_ids: Vec<ObjectId> = self.inner().get_pages().into_values().collect();

        if order.len() != page_ids.len() {
            return Err(Error::InvalidPageOrder);
        }

        let mut seen = vec![false; page_ids.len()];
        for &index in order {
            let slot = seen.get_mut(index).ok_or(Error::PageNotFound(index))?;
            if std::mem::replace(slot, true) {
                return Err(Error::InvalidPageOrder);
            }
        }

        let reordered_ids: Vec<ObjectId> = order.iter().map(|&index| page_ids[index]).collect();

        let mut result = self.inner().clone();
        let pages_root_id = result
            .catalog()
            .and_then(|catalog| catalog.get(b"Pages"))
            .and_then(Object::as_reference)
            .map_err(map_lopdf_error)?;
        let pages_dict = result
            .get_dictionary_mut(pages_root_id)
            .map_err(map_lopdf_error)?;
        pages_dict.set(
            "Kids",
            reordered_ids
                .into_iter()
                .map(Object::Reference)
                .collect::<Vec<_>>(),
        );

        Ok(Document::from_inner(result))
    }
}
