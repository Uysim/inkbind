//! `inkbind` is a helper library for working with PDF documents.
//!
//! The crate is being built incrementally. The current release provides the
//! shared error type and the module layout that later features build on:
//!
//! - [`document`] — loading a PDF and walking its object model.
//! - [`text`] — extracting text content from a page or document.
//! - [`metadata`] — reading document information such as title and author.
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

pub use error::{Error, Result};

/// The version of the `inkbind` crate, as reported by Cargo at build time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
