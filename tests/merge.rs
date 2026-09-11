//! Integration tests for `Document::merge` (Step 9).

use inkbind::{Document, Metadata};

fn fixture(name: &str) -> Document {
    Document::open(format!("tests/fixtures/{name}")).expect("fixture should load")
}

#[test]
fn merge_of_empty_slice_yields_zero_pages() {
    let merged = Document::merge(&[]).expect("merge should succeed");
    assert_eq!(merged.page_count(), 0);
}

#[test]
fn merge_of_single_document_preserves_its_pages() {
    let doc = fixture("two_pages.pdf");
    let merged = Document::merge(std::slice::from_ref(&doc)).expect("merge should succeed");

    assert_eq!(merged.page_count(), doc.page_count());
    assert_eq!(merged.text().unwrap(), doc.text().unwrap());
}

#[test]
fn merge_concatenates_pages_in_document_and_page_order() {
    let two_pages = fixture("two_pages.pdf");
    let minimal = fixture("minimal.pdf");
    let merged = Document::merge(&[two_pages, minimal]).expect("merge should succeed");

    assert_eq!(merged.page_count(), 3);
    assert!(merged.page(0).unwrap().text().unwrap().contains("Page One"));
    assert!(merged.page(1).unwrap().text().unwrap().contains("Page Two"));
    assert!(merged
        .page(2)
        .unwrap()
        .text()
        .unwrap()
        .contains("Hello Inkbind"));
}

#[test]
fn merge_respects_input_order_not_just_page_order_within_each_doc() {
    let minimal = fixture("minimal.pdf");
    let two_pages = fixture("two_pages.pdf");
    let merged = Document::merge(&[minimal, two_pages]).expect("merge should succeed");

    assert_eq!(merged.page_count(), 3);
    assert!(merged
        .page(0)
        .unwrap()
        .text()
        .unwrap()
        .contains("Hello Inkbind"));
    assert!(merged.page(1).unwrap().text().unwrap().contains("Page One"));
    assert!(merged.page(2).unwrap().text().unwrap().contains("Page Two"));
}

#[test]
fn merge_preserves_page_resources_for_image_extraction() {
    let two_pages = fixture("two_pages.pdf");
    let image_doc = fixture("image_flate.pdf");
    let merged = Document::merge(&[two_pages, image_doc]).expect("merge should succeed");

    assert_eq!(merged.page_count(), 3);
    let images = merged.page(2).unwrap().images().unwrap();
    assert_eq!(images.len(), 1);
    assert_eq!(images[0].width, 2);
    assert_eq!(images[0].height, 2);
}

#[test]
fn merge_does_not_propagate_per_document_metadata() {
    let doc = fixture("full_metadata.pdf");
    let merged = Document::merge(&[doc]).expect("merge should succeed");

    assert_eq!(merged.metadata().unwrap(), Metadata::default());
}
