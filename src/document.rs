//! Loading a PDF document and walking its object model.
//!
//! Document loading (this module) and page enumeration are implemented as of
//! Step 4, on top of `lopdf` — see
//! [ADR 0001](https://github.com/Uysim/inkbind/blob/main/docs/adr/0001-pdf-parsing-approach.md).
//! `lopdf` is an implementation detail: its types never appear in this
//! module's public API. Text extraction ([`crate::text`]) is implemented as
//! of Step 5; metadata extraction ([`crate::metadata`]) as of Step 6.
//! Serializing a document back to bytes or a file ([`Document::to_bytes`],
//! [`Document::save`]) is implemented as of Step 13, alongside
//! [`crate::writer`], which builds new documents from scratch.

use std::path::Path;
use std::sync::Arc;

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
    inner: Arc<lopdf::Document>,
}

impl Document {
    /// Opens a PDF document from a file path.
    ///
    /// Returns [`Error::Io`] if the file cannot be read, or
    /// [`Error::InvalidPdf`] if its contents cannot be parsed as a PDF.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let inner = lopdf::Document::load(path).map_err(map_lopdf_error)?;
        Ok(Document {
            inner: Arc::new(inner),
        })
    }

    /// Parses a PDF document from an in-memory byte buffer.
    ///
    /// Returns [`Error::InvalidPdf`] if the bytes cannot be parsed as a PDF.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let inner = lopdf::Document::load_mem(bytes).map_err(map_lopdf_error)?;
        Ok(Document {
            inner: Arc::new(inner),
        })
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
            Ok(Page {
                index,
                source: Arc::clone(&self.inner),
            })
        } else {
            Err(Error::PageNotFound(index))
        }
    }

    /// Serializes the document to PDF bytes, suitable for writing to a file
    /// or passing to [`Document::from_bytes`].
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut inner = self.inner().clone();
        let mut buffer = Vec::new();
        inner.save_to(&mut buffer)?;
        Ok(buffer)
    }

    /// Serializes the document and writes it to `path`.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut inner = self.inner().clone();
        inner.save(path)?;
        Ok(())
    }

    /// The underlying `lopdf` document, for use by sibling modules
    /// (`crate::text`, `crate::metadata`) that need to call `lopdf` APIs
    /// directly. Never exposed outside the crate — `lopdf` types don't cross
    /// the public API boundary.
    pub(crate) fn inner(&self) -> &lopdf::Document {
        &self.inner
    }

    /// Wraps an already-built `lopdf::Document` as an `inkbind::Document`,
    /// for sibling modules (`crate::merge`) that construct a new document
    /// rather than loading one from bytes.
    pub(crate) fn from_inner(inner: lopdf::Document) -> Self {
        Document {
            inner: Arc::new(inner),
        }
    }
}

/// Maps a `lopdf` error onto `inkbind`'s own error type, so `lopdf` error
/// types never cross the public API boundary. Used for both document
/// loading (`open`/`from_bytes`) and text extraction (`crate::text`).
pub(crate) fn map_lopdf_error(err: lopdf::Error) -> Error {
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
    pub(crate) source: Arc<lopdf::Document>,
}

impl Page {
    /// The page's 0-based index within its parent document.
    pub fn index(&self) -> usize {
        self.index
    }

    /// The page's 1-based page number, as used by `lopdf`'s page-tree APIs
    /// (`get_pages()`'s `BTreeMap` keys, `extract_text`'s `page_numbers`).
    pub(crate) fn page_number(&self) -> u32 {
        (self.index + 1) as u32
    }

    /// The page's `lopdf` object id, as used by `lopdf`'s resource-walking
    /// APIs (`get_page_resources`, `get_page_fonts`). `None` only if the
    /// page vanished from `source` after this `Page` was constructed, which
    /// cannot happen through the public API.
    pub(crate) fn page_id(&self) -> Option<lopdf::ObjectId> {
        self.source.get_pages().get(&self.page_number()).copied()
    }
}
