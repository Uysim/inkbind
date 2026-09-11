//! Reading document information such as title, author, and creation date.
//!
//! Implemented as of Step 6 on top of the PDF's `/Info` dictionary, reached
//! via `lopdf::Document::trailer`'s `/Info` reference. `/Info` values are
//! PDF "text strings": either PDFDocEncoded bytes or, when prefixed with the
//! `0xFE 0xFF` byte-order mark, UTF-16BE. This module detects the BOM and
//! decodes accordingly, falling back to a lossy UTF-8 read otherwise — exact
//! for the common ASCII case. `lopdf` is an implementation detail: its types
//! never appear in this module's public API.

use lopdf::Dictionary;

use crate::document::Document;
use crate::Result;

/// Document-level metadata read from a PDF's `/Info` dictionary.
///
/// All fields are optional because the PDF specification does not require
/// producers to populate them, and because the `/Info` dictionary itself is
/// optional.
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
    /// The document's creation date, as the raw PDF date string (e.g.
    /// `D:20240115093000+00'00'`). `inkbind` does not parse it into a
    /// structured date type.
    pub creation_date: Option<String>,
}

impl Document {
    /// Reads the document's `/Info` metadata.
    ///
    /// Returns [`Metadata::default`] (all fields `None`) when the `/Info`
    /// dictionary is missing entirely, since the PDF specification does not
    /// require producers to include one. A field within a present `/Info`
    /// dictionary is `None` when that field itself is absent.
    pub fn metadata(&self) -> Result<Metadata> {
        Ok(match self.info_dictionary() {
            Some(dict) => Metadata {
                title: info_string(dict, b"Title"),
                author: info_string(dict, b"Author"),
                subject: info_string(dict, b"Subject"),
                keywords: info_string(dict, b"Keywords"),
                creator: info_string(dict, b"Creator"),
                producer: info_string(dict, b"Producer"),
                creation_date: info_string(dict, b"CreationDate"),
            },
            None => Metadata::default(),
        })
    }

    fn info_dictionary(&self) -> Option<&Dictionary> {
        let info_id = self
            .inner()
            .trailer
            .get(b"Info")
            .ok()?
            .as_reference()
            .ok()?;
        self.inner().get_dictionary(info_id).ok()
    }
}

/// Reads and decodes a single `/Info` string field, or `None` if the key is
/// absent or is not a string.
fn info_string(dict: &Dictionary, key: &[u8]) -> Option<String> {
    let bytes = dict.get(key).ok()?.as_str().ok()?;
    Some(decode_info_string(bytes))
}

/// Decodes a PDF "text string": UTF-16BE (detected via the `0xFE 0xFF` BOM)
/// or, otherwise, PDFDocEncoding — approximated here as lossy UTF-8, which
/// is exact for the common ASCII case.
fn decode_info_string(bytes: &[u8]) -> String {
    if let [0xFE, 0xFF, rest @ ..] = bytes {
        let units: Vec<u16> = rest
            .chunks_exact(2)
            .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
            .collect();
        String::from_utf16_lossy(&units)
    } else {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::decode_info_string;

    #[test]
    fn decodes_ascii_bytes_as_utf8() {
        assert_eq!(decode_info_string(b"Inkbind"), "Inkbind");
    }

    #[test]
    fn decodes_utf16be_bytes_with_bom() {
        let mut bytes = vec![0xFE, 0xFF];
        bytes.extend("café".encode_utf16().flat_map(|u| u.to_be_bytes()));
        assert_eq!(decode_info_string(&bytes), "café");
    }

    #[test]
    fn odd_trailing_byte_after_bom_is_dropped_not_panicking() {
        // chunks_exact(2) silently ignores a trailing incomplete pair rather
        // than panicking on malformed input.
        let bytes = [0xFE, 0xFF, 0x00, 0x41, 0x00];
        assert_eq!(decode_info_string(&bytes), "A");
    }
}
