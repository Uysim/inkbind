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
//!
//! # Example
//!
//! ```
//! assert_eq!(inkbind::VERSION, env!("CARGO_PKG_VERSION"));
//! ```

pub mod document;
pub mod images;
pub mod metadata;
pub mod text;

mod error;

pub use document::{Document, Page};
pub use error::{Error, Result};
pub use images::{Image, ImageFormat};
pub use metadata::Metadata;

/// The version of the `inkbind` crate, as reported by Cargo at build time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
