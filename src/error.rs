//! The crate-wide error and result types.

use std::fmt;
use std::io;

/// A specialized [`Result`](std::result::Result) type for `inkbind` operations.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors that can occur while loading or manipulating a PDF document.
///
/// The enum is `#[non_exhaustive]`: future releases may add variants as more of
/// the library is implemented, so downstream `match` expressions should include
/// a wildcard arm.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The input could not be recognized or parsed as a PDF document.
    InvalidPdf(String),
    /// The operation targets a PDF feature that `inkbind` does not support yet.
    Unsupported(String),
    /// An I/O error occurred while reading or writing PDF data.
    Io(io::Error),
    /// The requested page index is out of range for the document.
    PageNotFound(usize),
    /// A rotation was requested that is not a multiple of 90 degrees.
    InvalidRotation(i32),
    /// The page order given to [`crate::Document::reorder`] is not a
    /// permutation of the document's page indices (wrong length, or a
    /// duplicate index).
    InvalidPageOrder,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidPdf(detail) => write!(f, "invalid PDF: {detail}"),
            Error::Unsupported(detail) => write!(f, "unsupported PDF feature: {detail}"),
            Error::Io(source) => write!(f, "I/O error: {source}"),
            Error::PageNotFound(index) => write!(f, "page index {index} out of range"),
            Error::InvalidRotation(degrees) => {
                write!(
                    f,
                    "rotation must be a multiple of 90 degrees, got {degrees}"
                )
            }
            Error::InvalidPageOrder => {
                write!(f, "page order is not a permutation of the document's pages")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(source) => Some(source),
            Error::InvalidPdf(_)
            | Error::Unsupported(_)
            | Error::PageNotFound(_)
            | Error::InvalidRotation(_)
            | Error::InvalidPageOrder => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(source: io::Error) -> Self {
        Error::Io(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_is_human_readable() {
        assert_eq!(
            Error::InvalidPdf("missing %PDF header".to_owned()).to_string(),
            "invalid PDF: missing %PDF header",
        );
        assert_eq!(
            Error::Unsupported("encrypted documents".to_owned()).to_string(),
            "unsupported PDF feature: encrypted documents",
        );
    }

    #[test]
    fn converts_from_io_error_and_exposes_source() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "no such file");
        let error = Error::from(io_error);

        assert!(matches!(error, Error::Io(_)));
        assert!(std::error::Error::source(&error).is_some());
        assert!(error.to_string().starts_with("I/O error: "));
    }

    #[test]
    fn non_io_variants_have_no_source() {
        let error = Error::Unsupported("x".to_owned());
        assert!(std::error::Error::source(&error).is_none());
    }

    #[test]
    fn page_not_found_displays_the_index() {
        assert_eq!(
            Error::PageNotFound(3).to_string(),
            "page index 3 out of range",
        );
        assert!(std::error::Error::source(&Error::PageNotFound(3)).is_none());
    }

    #[test]
    fn invalid_rotation_displays_the_degrees() {
        assert_eq!(
            Error::InvalidRotation(45).to_string(),
            "rotation must be a multiple of 90 degrees, got 45",
        );
        assert!(std::error::Error::source(&Error::InvalidRotation(45)).is_none());
    }

    #[test]
    fn invalid_page_order_is_human_readable() {
        assert_eq!(
            Error::InvalidPageOrder.to_string(),
            "page order is not a permutation of the document's pages",
        );
        assert!(std::error::Error::source(&Error::InvalidPageOrder).is_none());
    }
}
