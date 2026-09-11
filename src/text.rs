//! Extracting text content from a PDF page or document.
//!
//! Implemented as of Step 5 on top of `lopdf::Document::extract_text`, which
//! decodes each page's content stream and walks its text-showing operators
//! (`Tf`/`Tj`/`TJ`/`ET`) using the page's real font encodings — this gives
//! basic layout/ordering awareness (a newline after each `ET` block) without
//! `inkbind` needing its own content-stream walker. `lopdf` is an
//! implementation detail: its types never appear in this module's public
//! API.

use crate::document::{map_lopdf_error, Document, Page};
use crate::Result;

impl Document {
    /// Extracts and concatenates the text content of every page, in order.
    pub fn text(&self) -> Result<String> {
        let page_numbers: Vec<u32> = (1..=self.page_count() as u32).collect();
        self.inner()
            .extract_text(&page_numbers)
            .map_err(map_lopdf_error)
    }
}

impl Page {
    /// Extracts the text content of this page.
    pub fn text(&self) -> Result<String> {
        self.source
            .extract_text(&[self.page_number()])
            .map_err(map_lopdf_error)
    }
}
