//! Executable evidence for the Step 2 evaluation (see `docs/adr/0001-pdf-parsing-approach.md`).
//!
//! This spike exercises [`lopdf`] — the crate the ADR selects as inkbind's
//! low-level PDF parsing / object-model backend — against a tiny hand-built
//! fixture. Each assertion stands in for a capability the inkbind roadmap needs:
//!
//! | Assertion                     | Roadmap step it unblocks                  |
//! |-------------------------------|-------------------------------------------|
//! | parse header / `version`      | Step 4 — document loading                  |
//! | walk the page tree            | Step 4 / Step 10 — object model, split     |
//! | read the `/Info` dictionary   | Step 6 — metadata extraction               |
//! | decode a page content stream  | Step 5 — text extraction                   |
//! | re-serialize the document     | Step 9 / Step 13 — merge, writer           |
//!
//! `lopdf` is a **dev-dependency only** at this step: Step 2 evaluates and
//! proves the approach, Step 4 promotes it to a normal dependency behind
//! inkbind's own types. If this test ever fails to compile on the MSRV
//! toolchain, the pinned `Cargo.lock` in the repo root needs revisiting.

use std::path::Path;

use lopdf::{Document, Object};

fn load_fixture() -> Document {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("minimal.pdf");
    Document::load(&path).expect("fixture minimal.pdf should parse")
}

#[test]
fn lopdf_reports_the_pdf_version() {
    // The classic cross-reference table + `%PDF-1.7` header must round-trip
    // into a usable version string (Step 4).
    let doc = load_fixture();
    assert_eq!(doc.version, "1.7");
}

#[test]
fn lopdf_walks_the_page_tree() {
    // `get_pages` resolves Catalog -> Pages -> Kids; inkbind needs reliable
    // page enumeration for split / rotate / merge (Steps 9-11).
    let doc = load_fixture();
    let pages = doc.get_pages();
    assert_eq!(pages.len(), 1, "fixture has exactly one page");
    assert!(pages.contains_key(&1), "pages are keyed by 1-based number");
}

#[test]
fn lopdf_exposes_document_metadata() {
    // Step 6: title / producer / creation date live in the trailer's `/Info`
    // dictionary. Prove we can follow the reference and read a string entry.
    let doc = load_fixture();

    let info_id = match doc.trailer.get(b"Info") {
        Ok(Object::Reference(id)) => *id,
        other => panic!("expected /Info to be an indirect reference, got {other:?}"),
    };
    let info = doc
        .get_object(info_id)
        .and_then(Object::as_dict)
        .expect("/Info should resolve to a dictionary");

    let title = info.get(b"Title").and_then(Object::as_str).expect("/Title");
    assert_eq!(title, b"Inkbind Minimal Fixture");

    let producer = info
        .get(b"Producer")
        .and_then(Object::as_str)
        .expect("/Producer");
    assert_eq!(producer, b"inkbind gen_minimal_pdf.py");
}

#[test]
fn lopdf_decodes_a_page_content_stream() {
    // Step 5: text extraction starts from the decoded content stream. The
    // fixture stores it uncompressed, so the drawing operators must survive.
    let doc = load_fixture();
    let (_, &page_id) = doc.get_pages().iter().next().expect("one page");

    let content = doc
        .get_page_content(page_id)
        .expect("page content stream should decode");
    let rendered = String::from_utf8_lossy(&content);

    assert!(rendered.contains("Hello Inkbind"), "text operand preserved");
    assert!(rendered.contains("Tj"), "show-text operator preserved");
}

#[test]
fn lopdf_round_trips_the_document() {
    // Steps 9 & 13: merge and the basic writer both depend on serializing a
    // parsed document back to bytes without losing its structure.
    let mut doc = load_fixture();

    let mut buf = Vec::new();
    doc.save_to(&mut buf).expect("document should re-serialize");

    assert!(buf.starts_with(b"%PDF-"), "output keeps the PDF header");
    assert!(
        buf.windows(5).any(|w| w == b"%%EOF"),
        "output keeps the trailer marker"
    );

    // The serialized bytes must themselves be parseable again.
    let reloaded = Document::load_mem(&buf).expect("re-serialized bytes should parse");
    assert_eq!(reloaded.get_pages().len(), 1);
}
