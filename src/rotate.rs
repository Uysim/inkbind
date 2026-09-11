//! Rotating individual pages of a PDF document.
//!
//! Implemented as of Step 11. [`Document::rotate_page`] clones the source
//! document (the same technique [`crate::split`] uses) and rewrites only
//! the target page's own `/Rotate` entry — every other object, including
//! the page's `/Parent` chain, is left untouched. `/Rotate` is, per the PDF
//! spec, inheritable from an ancestor `/Pages` node when a page has none of
//! its own; [`Page::rotation`] does not walk that inheritance chain and
//! reads only the page's own dictionary, defaulting to `0` when absent —
//! a documented, non-blocking limitation, same class as prior steps'
//! inheritance notes. None of inkbind's fixtures rely on an inherited
//! `/Rotate`, so this is untested territory rather than a known-wrong
//! result. `lopdf` is an implementation detail: its types never appear in
//! this module's public API.

use crate::document::{map_lopdf_error, Document, Page};
use crate::{Error, Result};

impl Document {
    /// Rotates the page at `index` clockwise by `degrees`, relative to its
    /// current rotation. `degrees` must be a multiple of 90 (negative values
    /// and values beyond a full turn are normalized, e.g. `-90` and `450`
    /// both leave a page at `90` degrees more clockwise than it started).
    ///
    /// Returns [`Error::InvalidRotation`] if `degrees` is not a multiple of
    /// 90, or [`Error::PageNotFound`] if `index >= self.page_count()`.
    pub fn rotate_page(&self, index: usize, degrees: i32) -> Result<Document> {
        if degrees % 90 != 0 {
            return Err(Error::InvalidRotation(degrees));
        }

        let page_number = (index + 1) as u32;
        let page_id = *self
            .inner()
            .get_pages()
            .get(&page_number)
            .ok_or(Error::PageNotFound(index))?;

        let mut result = self.inner().clone();
        let page_dict = result
            .get_dictionary_mut(page_id)
            .map_err(map_lopdf_error)?;
        let current = page_dict
            .get(b"Rotate")
            .ok()
            .and_then(|object| object.as_i64().ok())
            .unwrap_or(0);
        let new_rotation = (current + degrees as i64).rem_euclid(360);
        page_dict.set("Rotate", new_rotation);

        Ok(Document::from_inner(result))
    }
}

impl Page {
    /// The page's rotation, in degrees clockwise, normalized to one of `0`,
    /// `90`, `180`, or `270`. Reflects only the page's own `/Rotate` entry;
    /// see this module's documentation for the inherited-`/Rotate` caveat.
    /// Defaults to `0` when the page has no `/Rotate` entry of its own.
    pub fn rotation(&self) -> i32 {
        self.page_id()
            .and_then(|id| self.source.get_dictionary(id).ok())
            .and_then(|dict| dict.get(b"Rotate").ok())
            .and_then(|object| object.as_i64().ok())
            .map(|degrees| degrees.rem_euclid(360) as i32)
            .unwrap_or(0)
    }
}
