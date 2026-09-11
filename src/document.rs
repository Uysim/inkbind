//! Loading a PDF document and walking its object model.
//!
//! This module currently defines the public API surface only (Step 3 — see
//! [ADR 0001](https://github.com/Uysim/inkbind/blob/main/docs/adr/0001-pdf-parsing-approach.md)).
//! [`Document::open`] and [`Document::from_bytes`] return
//! [`Error::Unsupported`] until Step 4 wires up the real `lopdf`-backed
//! parser behind these types.

use std::path::Path;

use crate::metadata::Metadata;
use crate::{Error, Result};

/// An in-memory representation of a PDF document.
///
/// `Document` is the entry point for reading a PDF's page tree, metadata, and
/// content. Instances are created with [`Document::open`] or
/// [`Document::from_bytes`].
///
/// ```no_run
/// # fn main() -> inkbind::Result<()> {
/// let doc = inkbind::Document::open("example.pdf")?;
/// println!("{} pages", doc.page_count());
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Document {
    _private: (),
}

impl Document {
    /// Opens a PDF document from a file path.
    ///
    /// This is currently a stub: it always returns [`Error::Unsupported`].
    /// Real parsing lands in Step 4.
    pub fn open<P: AsRef<Path>>(_path: P) -> Result<Self> {
        Err(Error::Unsupported("Document::open".to_owned()))
    }

    /// Parses a PDF document from an in-memory byte buffer.
    ///
    /// This is currently a stub: it always returns [`Error::Unsupported`].
    /// Real parsing lands in Step 4.
    pub fn from_bytes(_bytes: &[u8]) -> Result<Self> {
        Err(Error::Unsupported("Document::from_bytes".to_owned()))
    }

    /// Returns the number of pages in the document.
    ///
    /// Always `0` until document loading is implemented in Step 4.
    pub fn page_count(&self) -> usize {
        0
    }

    /// Returns the page at `index` (0-based).
    ///
    /// Returns [`Error::Unsupported`] for every index today, since no
    /// document ever has pages loaded yet ([`Document::page_count`] is
    /// always `0`).
    pub fn page(&self, index: usize) -> Result<Page> {
        if index < self.page_count() {
            Ok(Page { index })
        } else {
            Err(Error::Unsupported("Document::page".to_owned()))
        }
    }

    /// Reads the document's `/Info` metadata.
    ///
    /// This is currently a stub: it always returns [`Error::Unsupported`].
    /// Real metadata extraction lands in Step 6.
    pub fn metadata(&self) -> Result<Metadata> {
        Err(Error::Unsupported("Document::metadata".to_owned()))
    }
}

/// A single page within a [`Document`].
///
/// Pages are obtained from [`Document::page`].
#[derive(Debug)]
pub struct Page {
    index: usize,
}

impl Page {
    /// The page's 0-based index within its parent document.
    pub fn index(&self) -> usize {
        self.index
    }
}
