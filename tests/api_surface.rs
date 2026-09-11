//! Compile- and behavior-level checks for the public API sketched in Step 3.
//!
//! `Document::open` / `Document::from_bytes` / `Document::page_count` /
//! `Document::page` are real as of Step 4 (see
//! `docs/adr/0001-pdf-parsing-approach.md`); `Document::metadata` and
//! `Document::text` / `Page::text` remain documented stubs until Steps 5-6.
//! Deeper Step 4 behavior (multi-page enumeration, error cases) lives in
//! `tests/document_loading.rs` — these tests just pin the *shape* of the API
//! plus a minimal happy path.

use inkbind::{Document, Error, Metadata, Page, Result};

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
fn metadata_and_text_are_still_unsupported_stubs() {
    // `metadata`/`text` are Step 5/6 contracts; pin their still-stubbed
    // behavior so those steps' TDD red/green is unambiguous.
    let doc = Document::open("tests/fixtures/minimal.pdf").expect("fixture should parse");
    assert!(matches!(doc.metadata().unwrap_err(), Error::Unsupported(_)));
    assert!(matches!(doc.text().unwrap_err(), Error::Unsupported(_)));

    let page = doc.page(0).expect("fixture has one page");
    assert!(matches!(page.text().unwrap_err(), Error::Unsupported(_)));
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
}
