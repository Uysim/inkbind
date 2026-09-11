//! Step 4 — parsing PDF file structure (xref table, trailer, objects) into
//! an in-memory document model, and walking the resulting page tree.
//!
//! `tests/api_surface.rs` pins the public shape and a minimal happy path;
//! this file covers the object-model behavior in more depth: multi-page
//! enumeration/ordering, 0-based indexing at both ends of the range, and the
//! error cases `open`/`from_bytes`/`page` can hit.

use inkbind::{Document, Error};

#[test]
fn open_and_from_bytes_agree_on_page_count() {
    let from_path =
        Document::open("tests/fixtures/minimal.pdf").expect("open should parse the fixture");
    let bytes = std::fs::read("tests/fixtures/minimal.pdf").expect("fixture should be readable");
    let from_mem = Document::from_bytes(&bytes).expect("from_bytes should parse the same bytes");

    assert_eq!(from_path.page_count(), from_mem.page_count());
    assert_eq!(from_path.page_count(), 1);
}

#[test]
fn multi_page_document_reports_the_real_page_count() {
    let doc = Document::open("tests/fixtures/two_pages.pdf").expect("fixture should parse");
    assert_eq!(doc.page_count(), 2);
}

#[test]
fn pages_are_reachable_at_every_valid_0_based_index() {
    let doc = Document::open("tests/fixtures/two_pages.pdf").expect("fixture should parse");

    let first = doc.page(0).expect("index 0 should exist");
    assert_eq!(first.index(), 0);

    let last = doc
        .page(doc.page_count() - 1)
        .expect("last valid index should exist");
    assert_eq!(last.index(), 1);
}

#[test]
fn page_out_of_range_is_reported_as_page_not_found() {
    let doc = Document::open("tests/fixtures/minimal.pdf").expect("fixture should parse");

    let err = doc.page(doc.page_count()).unwrap_err();
    assert!(matches!(err, Error::PageNotFound(1)));
}

#[test]
fn from_bytes_rejects_input_that_is_not_a_pdf() {
    let err = Document::from_bytes(b"not a pdf").unwrap_err();
    assert!(matches!(err, Error::InvalidPdf(_)));
}

#[test]
fn open_reports_io_error_for_a_missing_file() {
    let err = Document::open("tests/fixtures/does-not-exist.pdf").unwrap_err();
    assert!(matches!(err, Error::Io(_)));
}
