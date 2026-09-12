//! Splitting a PDF document into individual pages or arbitrary page ranges.
//!
//! Implemented as of Step 10. The inverse of [`crate::merge`]: where `merge`
//! combines N documents into one, [`Document::split`] derives N new
//! documents from one. Because every output page comes from the same,
//! internally-consistent source document, no object-id renumbering is
//! needed — each output is a full clone of the source with only its root
//! `/Pages` dictionary's `/Kids` and `/Count` narrowed to the selected
//! pages. Every page's own `/Parent` chain, and any intermediate `/Pages`
//! ancestor it inherits `/Resources`/`/MediaBox` from, is left completely
//! untouched — `lopdf`'s page-tree traversal (`get_pages`/`page_iter`) only
//! walks `/Kids` downward from `/Root` and never consults `/Parent`, so a
//! page reached solely through the root's rewritten `/Kids` still resolves
//! inherited attributes correctly by walking its original, unmodified
//! `/Parent` chain. This is an improvement over `merge`, which must
//! flatten every input's page tree to a single level and can therefore
//! lose an intermediate ancestor's inherited attributes. Pages outside the
//! selection remain in the cloned object map as harmless orphans (same
//! precedent as `merge`'s orphaned later-`/Pages` dicts): no writer exists
//! yet to bloat, and no public read API reaches a page except by walking
//! `/Kids` from `/Root`. Unlike `merge`, whole-document properties such as
//! `/Info` metadata are untouched and so are preserved in every output.
//! `lopdf` is an implementation detail: its types never appear in this
//! module's public API.

use std::ops::Range;

use lopdf::{Object, ObjectId};

use crate::document::{map_lopdf_error, Document};
use crate::{Error, Result};

impl Document {
    /// Splits the document into one new [`Document`] per entry in `ranges`,
    /// each containing the pages in that half-open, 0-based page-index
    /// range, in order. A range of length 1 selects a single page; ranges
    /// need not be contiguous with each other, and may overlap.
    ///
    /// A range whose `start >= end` yields a valid, zero-page document —
    /// the same precedent [`Document::merge`] set for an empty input slice.
    ///
    /// Returns [`Error::PageNotFound`] if any range's `end` exceeds
    /// [`Document::page_count`].
    ///
    /// ```
    /// let doc = inkbind::Document::open("tests/fixtures/two_pages.pdf")?;
    ///
    /// let mut parts = doc.split(&[0..1, 1..2])?;
    /// assert_eq!(parts.len(), 2);
    /// let second = parts.pop().unwrap();
    /// let first = parts.pop().unwrap();
    /// assert_eq!(first.page_count(), 1);
    /// assert!(first.page(0)?.text()?.contains("Page One"));
    /// assert!(second.page(0)?.text()?.contains("Page Two"));
    /// # Ok::<(), inkbind::Error>(())
    /// ```
    pub fn split(&self, ranges: &[Range<usize>]) -> Result<Vec<Document>> {
        let page_ids: Vec<ObjectId> = self.inner().get_pages().into_values().collect();

        ranges
            .iter()
            .map(|range| {
                if range.end > page_ids.len() {
                    return Err(Error::PageNotFound(range.end));
                }
                let selected = if range.start >= range.end {
                    &[][..]
                } else {
                    &page_ids[range.start..range.end]
                };
                build_split_document(self.inner(), selected)
            })
            .collect()
    }
}

/// Clones `source` and narrows its `/Pages` dictionary to only `selected`.
fn build_split_document(source: &lopdf::Document, selected: &[ObjectId]) -> Result<Document> {
    let mut result = source.clone();

    let pages_root_id = result
        .catalog()
        .and_then(|catalog| catalog.get(b"Pages"))
        .and_then(Object::as_reference)
        .map_err(map_lopdf_error)?;

    let pages_dict = result
        .get_dictionary_mut(pages_root_id)
        .map_err(map_lopdf_error)?;
    pages_dict.set("Count", selected.len() as i64);
    pages_dict.set(
        "Kids",
        selected
            .iter()
            .copied()
            .map(Object::Reference)
            .collect::<Vec<_>>(),
    );

    Ok(Document::from_inner(result))
}
