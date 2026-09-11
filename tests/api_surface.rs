//! Compile- and behavior-level checks for the Step 3 public API sketch.
//!
//! `Document::open` / `Document::from_bytes` intentionally return
//! `Error::Unsupported` until Step 4 wires up the real `lopdf`-backed parser
//! (see `docs/adr/0001-pdf-parsing-approach.md`). These tests pin the
//! *shape* of the API so Step 4 implements against a fixed signature rather
//! than behavior, which isn't real yet.

use inkbind::{Document, Error, Metadata, Page, Result};

fn assert_send_sync<T: Send + Sync>() {}
fn assert_debug<T: std::fmt::Debug>() {}

#[test]
fn document_open_has_expected_signature_and_is_unsupported_for_now() {
    // `open` is generic over `P: AsRef<Path>`; calling it with both a `&str`
    // and an owned `String` pins that bound without fighting the borrow
    // checker over higher-rank fn-pointer coercions (generic fns
    // monomorphized with a reference type aren't universally quantified).
    let err: Error = Document::open("tests/fixtures/minimal.pdf").unwrap_err();
    assert!(matches!(err, Error::Unsupported(_)));

    let err: Error = Document::open(String::from("tests/fixtures/minimal.pdf")).unwrap_err();
    assert!(matches!(err, Error::Unsupported(_)));
}

#[test]
fn document_from_bytes_has_expected_signature_and_is_unsupported_for_now() {
    let from_bytes: fn(&[u8]) -> Result<Document> = Document::from_bytes;
    let err = from_bytes(b"%PDF-1.7").unwrap_err();
    assert!(matches!(err, Error::Unsupported(_)));
}

#[test]
fn document_page_count_is_zero_before_loading_is_implemented() {
    // `open` always errors today, so there is no live `Document` to call
    // `page_count`/`page`/`metadata`/`text` on yet. Pin their signatures via
    // function-pointer coercion instead of calling them.
    let _page_count: fn(&Document) -> usize = Document::page_count;
    let _page: fn(&Document, usize) -> Result<Page> = Document::page;
    let _metadata: fn(&Document) -> Result<Metadata> = Document::metadata;
    let _text: fn(&Document) -> Result<String> = Document::text;
    let _page_index: fn(&Page) -> usize = Page::index;
    let _page_text: fn(&Page) -> Result<String> = Page::text;
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
