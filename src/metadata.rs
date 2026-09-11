//! Reading document information such as title, author, and creation date.
//!
//! This module currently defines the public API surface only (Step 3).
//! [`Document::metadata`](crate::Document::metadata) returns
//! [`crate::Error::Unsupported`] until Step 6 implements real `/Info`
//! dictionary extraction.

/// Document-level metadata read from a PDF's `/Info` dictionary.
///
/// All fields are optional because the PDF specification does not require
/// producers to populate them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Metadata {
    /// The document's title.
    pub title: Option<String>,
    /// The name of the person who created the document.
    pub author: Option<String>,
    /// The subject of the document.
    pub subject: Option<String>,
    /// Keywords associated with the document.
    pub keywords: Option<String>,
    /// The application that created the original document.
    pub creator: Option<String>,
    /// The application that produced the PDF from the original document.
    pub producer: Option<String>,
}
