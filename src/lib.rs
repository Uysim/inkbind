//! `inkbind` is a helper library for working with PDF documents.
//!
//! The crate is being built incrementally.
//!
//! - [`document`] — loading a PDF and walking its object model. Implemented
//!   as of Step 4.
//! - [`text`] — extracting text content from a page or document. Implemented
//!   as of Step 5.
//! - [`metadata`] — reading document information such as title and author
//!   ([`Document::metadata`]). Implemented as of Step 6.
//! - [`images`] — extracting embedded raster images from a page's resource
//!   dictionary ([`Document::images`], [`Page::images`]). Implemented as of
//!   Step 8.
//! - [`merge`] — combining multiple documents into one, preserving page
//!   order ([`Document::merge`]). Implemented as of Step 9.
//! - [`split`] — deriving new documents from individual pages or arbitrary
//!   page ranges of an existing document ([`Document::split`]). Implemented
//!   as of Step 10.
//! - [`rotate`] — rotating an individual page ([`Document::rotate_page`],
//!   [`Page::rotation`]). Implemented as of Step 11.
//! - [`reorder`] — reordering a document's page sequence
//!   ([`Document::reorder`]). Implemented as of Step 11.
//! - [`watermark`] — stamping a text watermark onto every page
//!   ([`Document::watermark_text`], [`WatermarkOptions`]). Implemented as of
//!   Step 12.
//! - [`writer`] — building a new document from scratch
//!   ([`DocumentBuilder`], [`PageSpec`], [`TextLine`]), plus serializing any
//!   document back to bytes or a file ([`Document::to_bytes`],
//!   [`Document::save`]). Implemented as of Step 13.
//!
//! # Example
//!
//! Build a one-page document from scratch, serialize it, and read its text
//! back out — see each module's own docs for the rest of the API surface
//! (opening an existing PDF, extracting metadata/images, merging, etc.).
//!
//! ```
//! use inkbind::{Document, DocumentBuilder, PageSpec, TextLine};
//!
//! let mut builder = DocumentBuilder::new();
//! builder.add_page(PageSpec::default().with_line(TextLine {
//!     text: "Hello, world!".to_owned(),
//!     x: 72.0,
//!     y: 720.0,
//!     font_size: 24.0,
//! }));
//! let doc = builder.build()?;
//!
//! let bytes = doc.to_bytes()?;
//! let reopened = Document::from_bytes(&bytes)?;
//! assert!(reopened.page(0)?.text()?.contains("Hello, world!"));
//! # Ok::<(), inkbind::Error>(())
//! ```

pub mod document;
pub mod images;
pub mod merge;
pub mod metadata;
pub mod reorder;
pub mod rotate;
pub mod split;
pub mod text;
pub mod watermark;
pub mod writer;

mod error;

pub use document::{Document, Page};
pub use error::{Error, Result};
pub use images::{Image, ImageFormat};
pub use metadata::Metadata;
pub use watermark::WatermarkOptions;
pub use writer::{DocumentBuilder, PageSpec, TextLine, LETTER_SIZE};

/// The version of the `inkbind` crate, as reported by Cargo at build time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
