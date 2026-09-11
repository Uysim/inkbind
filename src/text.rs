//! Extracting text content from a PDF page or document.
//!
//! This module currently defines the public API surface only (Step 3). Both
//! [`Document::text`] and [`Page::text`] return [`Error::Unsupported`] until
//! Step 5 implements real content-stream decoding.

use crate::document::{Document, Page};
use crate::{Error, Result};

impl Document {
    /// Extracts and concatenates the text content of every page, in order.
    ///
    /// This is currently a stub: it always returns [`Error::Unsupported`].
    /// Real extraction lands in Step 5.
    pub fn text(&self) -> Result<String> {
        Err(Error::Unsupported("Document::text".to_owned()))
    }
}

impl Page {
    /// Extracts the text content of this page.
    ///
    /// This is currently a stub: it always returns [`Error::Unsupported`].
    /// Real extraction lands in Step 5.
    pub fn text(&self) -> Result<String> {
        Err(Error::Unsupported("Page::text".to_owned()))
    }
}
