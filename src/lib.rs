//! `inkbind` is a helper library for working with PDF documents.
//!
//! The crate is being built incrementally. The current release sketches the
//! public API surface ([`Document`], [`Page`], [`Metadata`]) as documented
//! stubs; every operation that requires real PDF parsing returns
//! [`Error::Unsupported`] until the corresponding roadmap step lands:
//!
//! - [`document`] — loading a PDF and walking its object model (Step 4).
//! - [`text`] — extracting text content from a page or document (Step 5).
//! - [`metadata`] — reading document information such as title and author
//!   (Step 6).
//!
//! # Example
//!
//! ```
//! assert_eq!(inkbind::VERSION, env!("CARGO_PKG_VERSION"));
//! ```

pub mod document;
pub mod metadata;
pub mod text;

mod error;

pub use document::{Document, Page};
pub use error::{Error, Result};
pub use metadata::Metadata;

/// The version of the `inkbind` crate, as reported by Cargo at build time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
