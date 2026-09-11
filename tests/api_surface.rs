//! Compile- and behavior-level checks for the public API sketched in Step 3.
//!
//! `Document::open` / `Document::from_bytes` / `Document::page_count` /
//! `Document::page` are real as of Step 4 (see
//! `docs/adr/0001-pdf-parsing-approach.md`); `Document::text` / `Page::text`
//! are real as of Step 5; `Document::metadata` is real as of Step 6. Deeper
//! behavior for the implemented methods lives in `tests/document_loading.rs`
//! (Step 4), `tests/text_extraction.rs` (Step 5), and
//! `tests/metadata_extraction.rs` (Step 6) — these tests just pin the
//! *shape* of the API plus a minimal happy path.

use inkbind::{Document, Metadata, Page, Result};

fn assert_send_sync<T: Send + Sync>() {}
fn assert_debug<T: std::fmt::Debug>() {}

#[test]
fn document_open_has_expected_signature_and_parses_a_valid_pdf() {
    // `open` is generic over `P: AsRef<Path>`; calling it with both a `&str`
    // and an owned `String` pins that bound without fighting the borrow
    // checker over higher-rank fn-pointer coercions (generic fns
    // monomorphized with a reference type aren't universally quantified).
    let doc = Document::open("tests/fixtures/minimal.pdf").expect("fixture should parse");
    assert_eq!(doc.page_count(), 1);

    let doc =
        Document::open(String::from("tests/fixtures/minimal.pdf")).expect("fixture should parse");
    assert_eq!(doc.page_count(), 1);
}

#[test]
fn document_from_bytes_has_expected_signature_and_parses_a_valid_pdf() {
    let from_bytes: fn(&[u8]) -> Result<Document> = Document::from_bytes;
    let bytes = std::fs::read("tests/fixtures/minimal.pdf").expect("fixture should be readable");
    let doc = from_bytes(&bytes).expect("fixture bytes should parse");
    assert_eq!(doc.page_count(), 1);
}

#[test]
fn document_and_page_signatures_are_pinned() {
    let _page_count: fn(&Document) -> usize = Document::page_count;
    let _page: fn(&Document, usize) -> Result<Page> = Document::page;
    let _metadata: fn(&Document) -> Result<Metadata> = Document::metadata;
    let _text: fn(&Document) -> Result<String> = Document::text;
    let _page_index: fn(&Page) -> usize = Page::index;
    let _page_text: fn(&Page) -> Result<String> = Page::text;
}

#[test]
fn metadata_reads_the_info_dictionary() {
    // `metadata`'s full extraction behavior (all fields, partial fields,
    // encoding, missing /Info) lives in `tests/metadata_extraction.rs`;
    // this is just the happy-path shape pin.
    let doc = Document::open("tests/fixtures/minimal.pdf").expect("fixture should parse");
    let meta = doc.metadata().expect("metadata should be readable");
    assert_eq!(meta.title.as_deref(), Some("Inkbind Minimal Fixture"));
}

#[test]
fn document_and_page_are_send_sync_and_debug() {
    assert_send_sync::<Document>();
    assert_send_sync::<Page>();
    assert_debug::<Document>();
    assert_debug::<Page>();
}

#[test]
fn metadata_is_plain_data() {
    assert_send_sync::<Metadata>();
    assert_debug::<Metadata>();

    let meta = Metadata::default();
    assert_eq!(meta, Metadata::default());
    assert_eq!(meta.clone().title, None);
    assert_eq!(meta.creation_date, None);
}
