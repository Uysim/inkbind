//! Integration tests for `Document::reorder` (Step 11).
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
fn reorder_reverses_page_order() {
    let doc = fixture("two_pages.pdf");
    let reordered = doc.reorder(&[1, 0]).expect("reorder should succeed");

    assert_eq!(reordered.page_count(), 2);
    assert!(reordered
        .page(0)
        .unwrap()
        .text()
        .unwrap()
        .contains("Page Two"));
    assert!(reordered
        .page(1)
        .unwrap()
        .text()
        .unwrap()
        .contains("Page One"));
}

#[test]
fn reorder_with_identity_order_is_a_no_op() {
    let doc = fixture("two_pages.pdf");
    let reordered = doc.reorder(&[0, 1]).expect("reorder should succeed");

    assert!(reordered
        .page(0)
        .unwrap()
        .text()
        .unwrap()
        .contains("Page One"));
    assert!(reordered
        .page(1)
        .unwrap()
        .text()
        .unwrap()
        .contains("Page Two"));
}

#[test]
fn reorder_wrong_length_is_an_error() {
    let doc = fixture("two_pages.pdf");
    let err = doc.reorder(&[0]).unwrap_err();
    assert!(matches!(err, Error::InvalidPageOrder));
}

#[test]
fn reorder_duplicate_index_is_an_error() {
    let doc = fixture("two_pages.pdf");
    let err = doc.reorder(&[0, 0]).unwrap_err();
    assert!(matches!(err, Error::InvalidPageOrder));
}

#[test]
fn reorder_out_of_range_index_is_an_error() {
    let doc = fixture("two_pages.pdf");
    let err = doc.reorder(&[0, 5]).unwrap_err();
    assert!(matches!(err, Error::PageNotFound(5)));
}

#[test]
fn reorder_of_a_zero_page_document_with_empty_order_succeeds() {
    let doc = fixture("two_pages.pdf");
    let empty = doc.split(&[1..1]).expect("split should succeed").remove(0);

    let reordered = empty.reorder(&[]).expect("reorder should succeed");
    assert_eq!(reordered.page_count(), 0);
}
