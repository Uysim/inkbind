//! Loading a PDF document and walking its object model.
//!
//! Document loading (this module) and page enumeration are implemented as of
//! Step 4, on top of `lopdf` — see
//! [ADR 0001](https://github.com/Uysim/inkbind/blob/main/docs/adr/0001-pdf-parsing-approach.md).
//! `lopdf` is an implementation detail: its types never appear in this
//! module's public API. [`Document::metadata`] and text extraction
//! ([`crate::text`]) remain documented stubs until Steps 5-6.

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
    inner: lopdf::Document,
}

impl Document {
    /// Opens a PDF document from a file path.
    ///
    /// Returns [`Error::Io`] if the file cannot be read, or
    /// [`Error::InvalidPdf`] if its contents cannot be parsed as a PDF.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let inner = lopdf::Document::load(path).map_err(map_load_error)?;
        Ok(Document { inner })
    }

    /// Parses a PDF document from an in-memory byte buffer.
    ///
    /// Returns [`Error::InvalidPdf`] if the bytes cannot be parsed as a PDF.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let inner = lopdf::Document::load_mem(bytes).map_err(map_load_error)?;
        Ok(Document { inner })
    }

    /// Returns the number of pages in the document.
    pub fn page_count(&self) -> usize {
        self.inner.get_pages().len()
    }

    /// Returns the page at `index` (0-based, in document order).
    ///
    /// Returns [`Error::PageNotFound`] if `index >= self.page_count()`.
    pub fn page(&self, index: usize) -> Result<Page> {
        if index < self.page_count() {
            Ok(Page { index })
        } else {
            Err(Error::PageNotFound(index))
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

/// Maps a `lopdf` load error onto `inkbind`'s own error type, so `lopdf`
/// error types never cross the public API boundary.
fn map_load_error(err: lopdf::Error) -> Error {
    match err {
        lopdf::Error::IO(io_err) => Error::Io(io_err),
        other => Error::InvalidPdf(other.to_string()),
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
