//! Step 5 — extracting text content from a page or whole document.
//!
//! `tests/api_surface.rs` pins the public shape (`Document::text` and
//! `Page::text` both `fn(&self) -> Result<String>`); this file covers the
//! extraction behavior itself: per-page content, ordering across pages,
//! ordering within a single page's multiple text blocks, and the
//! no-text-on-the-page edge case.

use inkbind::Document;

#[test]
fn page_text_returns_the_page_content() {
    let doc = Document::open("tests/fixtures/minimal.pdf").expect("fixture should parse");
    let page = doc.page(0).expect("fixture has one page");

    assert_eq!(
        page.text().expect("page should have text"),
        "Hello Inkbind\n"
    );
}

#[test]
fn document_text_matches_the_single_page_for_a_one_page_document() {
    let doc = Document::open("tests/fixtures/minimal.pdf").expect("fixture should parse");

    assert_eq!(
        doc.text().expect("document should have text"),
        "Hello Inkbind\n"
    );
}

#[test]
fn page_text_is_ordered_correctly_across_multiple_pages() {
    let doc = Document::open("tests/fixtures/two_pages.pdf").expect("fixture should parse");

    let first = doc.page(0).expect("index 0 should exist");
    let second = doc.page(1).expect("index 1 should exist");

    assert_eq!(first.text().expect("page 0 should have text"), "Page One\n");
    assert_eq!(
        second.text().expect("page 1 should have text"),
        "Page Two\n"
    );
}

#[test]
fn document_text_concatenates_every_page_in_order() {
    let doc = Document::open("tests/fixtures/two_pages.pdf").expect("fixture should parse");

    assert_eq!(
        doc.text().expect("document should have text"),
        "Page One\nPage Two\n"
    );
}

#[test]
fn page_with_no_text_extracts_as_an_empty_string() {
    let doc = Document::open("tests/fixtures/no_text.pdf").expect("fixture should parse");
    let page = doc.page(0).expect("fixture has one page");

    assert_eq!(page.text().expect("empty page should not error"), "");
    assert_eq!(doc.text().expect("empty document should not error"), "");
}

#[test]
fn multiple_text_blocks_on_one_page_are_joined_in_order() {
    let doc = Document::open("tests/fixtures/multiline.pdf").expect("fixture should parse");
    let page = doc.page(0).expect("fixture has one page");

    assert_eq!(
        page.text().expect("page should have text"),
        "First Line\nSecond Line\n"
    );
    assert_eq!(
        doc.text().expect("document should have text"),
        "First Line\nSecond Line\n"
    );
}
