//! Integration tests for `Document::rotate_page` and `Page::rotation` (Step 11).

use inkbind::{Document, Error};

fn fixture(name: &str) -> Document {
    Document::open(format!("tests/fixtures/{name}")).expect("fixture should load")
}

#[test]
fn page_rotation_defaults_to_zero() {
    let doc = fixture("minimal.pdf");
    assert_eq!(doc.page(0).unwrap().rotation(), 0);
}

#[test]
fn rotate_page_sets_the_rotation() {
    let doc = fixture("minimal.pdf");
    let rotated = doc.rotate_page(0, 90).expect("rotate should succeed");
    assert_eq!(rotated.page(0).unwrap().rotation(), 90);
}

#[test]
fn rotate_page_is_relative_to_the_existing_rotation() {
    let doc = fixture("minimal.pdf");
    let once = doc.rotate_page(0, 90).expect("rotate should succeed");
    let twice = once.rotate_page(0, 90).expect("rotate should succeed");
    assert_eq!(twice.page(0).unwrap().rotation(), 180);
}

#[test]
fn rotate_page_normalizes_negative_degrees() {
    let doc = fixture("minimal.pdf");
    let rotated = doc.rotate_page(0, -90).expect("rotate should succeed");
    assert_eq!(rotated.page(0).unwrap().rotation(), 270);
}

#[test]
fn rotate_page_normalizes_degrees_beyond_a_full_turn() {
    let doc = fixture("minimal.pdf");
    let rotated = doc.rotate_page(0, 450).expect("rotate should succeed");
    assert_eq!(rotated.page(0).unwrap().rotation(), 90);
}

#[test]
fn rotate_page_rejects_degrees_not_a_multiple_of_90() {
    let doc = fixture("minimal.pdf");
    let err = doc.rotate_page(0, 45).unwrap_err();
    assert!(matches!(err, Error::InvalidRotation(45)));
}

#[test]
fn rotate_page_out_of_range_index_is_an_error() {
    let doc = fixture("minimal.pdf");
    let err = doc.rotate_page(1, 90).unwrap_err();
    assert!(matches!(err, Error::PageNotFound(1)));
}

#[test]
fn rotate_page_does_not_affect_other_pages() {
    let doc = fixture("two_pages.pdf");
    let rotated = doc.rotate_page(0, 90).expect("rotate should succeed");
    assert_eq!(rotated.page(0).unwrap().rotation(), 90);
    assert_eq!(rotated.page(1).unwrap().rotation(), 0);
}

#[test]
fn rotate_page_preserves_page_text() {
    let doc = fixture("two_pages.pdf");
    let rotated = doc.rotate_page(1, 180).expect("rotate should succeed");
    assert!(rotated
        .page(1)
        .unwrap()
        .text()
        .unwrap()
        .contains("Page Two"));
}
