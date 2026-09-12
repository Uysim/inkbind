//! Step 14 — a fixture generator for "large" (multi-hundred-page) documents,
//! used by `benches/large_pdf.rs` to benchmark parsing/extraction. Generated
//! in-memory with the existing `DocumentBuilder` API rather than checked
//! into `tests/fixtures/` as a large binary. This test verifies the
//! generator itself is correct — right page count, page-isolated text,
//! survives a real serialize/reparse round trip — before it's trusted as a
//! benchmark input.

use inkbind::{Document, DocumentBuilder, PageSpec, TextLine};

const PAGE_COUNT: usize = 300;
const LINES_PER_PAGE: usize = 10;

fn build_large_document(page_count: usize, lines_per_page: usize) -> Document {
    let mut builder = DocumentBuilder::new();
    for page_index in 0..page_count {
        let mut page = PageSpec::default();
        for line_index in 0..lines_per_page {
            page = page.with_line(TextLine {
                text: format!("Page {page_index} line {line_index}"),
                x: 72.0,
                y: 720.0 - (line_index as f32 * 20.0),
                font_size: 12.0,
            });
        }
        builder.add_page(page);
    }
    builder.build().expect("large document should build")
}

#[test]
fn large_document_has_the_requested_page_count() {
    let doc = build_large_document(PAGE_COUNT, LINES_PER_PAGE);
    assert_eq!(doc.page_count(), PAGE_COUNT);
}

#[test]
fn large_document_pages_keep_independent_text() {
    let doc = build_large_document(PAGE_COUNT, LINES_PER_PAGE);

    let first = doc.page(0).unwrap().text().unwrap();
    let last = doc.page(PAGE_COUNT - 1).unwrap().text().unwrap();
    assert!(first.contains("Page 0 line 0"), "got {first:?}");
    assert!(
        !first.contains(&format!("Page {}", PAGE_COUNT - 1)),
        "first page leaked last page's text: {first:?}"
    );
    assert!(
        last.contains(&format!("Page {} line 0", PAGE_COUNT - 1)),
        "got {last:?}"
    );
    assert!(
        !last.contains("Page 0 line"),
        "last page leaked first page's text: {last:?}"
    );
}

#[test]
fn large_document_round_trips_through_bytes() {
    let doc = build_large_document(PAGE_COUNT, LINES_PER_PAGE);
    let bytes = doc.to_bytes().expect("serialization should succeed");
    let reopened = Document::from_bytes(&bytes).expect("reparsing should succeed");

    assert_eq!(reopened.page_count(), PAGE_COUNT);
    let text = reopened
        .text()
        .expect("full-document text extraction should succeed");
    assert!(text.contains("Page 0 line 0"), "got start of {text:?}");
    assert!(
        text.contains(&format!(
            "Page {} line {}",
            PAGE_COUNT - 1,
            LINES_PER_PAGE - 1
        )),
        "missing last page's last line"
    );
}
