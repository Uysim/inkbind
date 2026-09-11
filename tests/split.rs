//! Integration tests for `Document::split` (Step 10).
//!
//! A single-element `&[start..end]` is `Document::split`'s ordinary
//! single-range call shape, not the `[value; count]` repeat expression
//! clippy's `single_range_in_vec_init` lint suspects it might be confused
//! with — allowed crate-wide for this file rather than at every call site.
#![allow(clippy::single_range_in_vec_init)]

use inkbind::{Document, Error};

fn fixture(name: &str) -> Document {
    Document::open(format!("tests/fixtures/{name}")).expect("fixture should load")
}

#[test]
fn split_out_of_range_end_is_an_error() {
    let doc = fixture("two_pages.pdf");
    let err = doc.split(&[0..3]).unwrap_err();
    assert!(matches!(err, Error::PageNotFound(3)));
}

#[test]
fn split_single_page_range_yields_a_one_page_document() {
    let doc = fixture("two_pages.pdf");
    let parts = doc.split(&[1..2]).expect("split should succeed");

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].page_count(), 1);
    assert!(parts[0].text().unwrap().contains("Page Two"));
}

#[test]
fn split_into_individual_pages_preserves_each_pages_text_and_metadata() {
    let doc = fixture("two_pages.pdf");
    let expected_metadata = doc.metadata().unwrap();
    let parts = doc.split(&[0..1, 1..2]).expect("split should succeed");

    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].page_count(), 1);
    assert!(parts[0].text().unwrap().contains("Page One"));
    assert_eq!(parts[1].page_count(), 1);
    assert!(parts[1].text().unwrap().contains("Page Two"));

    // Splitting, unlike merging, keeps the whole document's `/Info` intact —
    // each output still carries the source document's metadata.
    assert_eq!(parts[0].metadata().unwrap(), expected_metadata);
    assert_eq!(parts[1].metadata().unwrap(), expected_metadata);
}

#[test]
fn split_multi_page_range_preserves_contiguous_page_order() {
    let two_pages = fixture("two_pages.pdf");
    let minimal = fixture("minimal.pdf");
    let combined = Document::merge(&[two_pages, minimal]).expect("merge should succeed");

    let parts = combined.split(&[0..2]).expect("split should succeed");

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].page_count(), 2);
    assert!(parts[0]
        .page(0)
        .unwrap()
        .text()
        .unwrap()
        .contains("Page One"));
    assert!(parts[0]
        .page(1)
        .unwrap()
        .text()
        .unwrap()
        .contains("Page Two"));
}

#[test]
fn split_empty_range_yields_a_valid_zero_page_document() {
    let doc = fixture("two_pages.pdf");
    let parts = doc.split(&[1..1]).expect("split should succeed");

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].page_count(), 0);
}

#[test]
fn split_preserves_page_resources_for_image_extraction() {
    let doc = fixture("image_flate.pdf");
    let parts = doc.split(&[0..1]).expect("split should succeed");

    let images = parts[0].page(0).unwrap().images().unwrap();
    assert_eq!(images.len(), 1);
    assert_eq!(images[0].width, 2);
    assert_eq!(images[0].height, 2);
}

#[test]
fn split_with_no_ranges_yields_no_documents() {
    let doc = fixture("two_pages.pdf");
    let parts = doc.split(&[]).expect("split should succeed");
    assert!(parts.is_empty());
}
