//! Step 8 — extracting embedded raster images from a page's resource
//! dictionary.
//!
//! `tests/api_surface.rs` pins the public shape (`Document::images` and
//! `Page::images`, both `fn(&self) -> Result<Vec<Image>>`); this file
//! covers the extraction behavior itself: decompressing a `/FlateDecode`
//! image to its exact raw pixel bytes, passing a `/DCTDecode` image's
//! bytes through unchanged, surfacing an unsupported filter as an error,
//! and the no-images-on-the-page edge case.

use inkbind::{Document, Error, ImageFormat};

#[test]
fn flate_image_decompresses_to_its_exact_raw_pixel_bytes() {
    let doc = Document::open("tests/fixtures/image_flate.pdf").expect("fixture should parse");
    let page = doc.page(0).expect("fixture has one page");

    let images = page.images().expect("image should decode");
    assert_eq!(images.len(), 1);

    let image = &images[0];
    assert_eq!(image.width, 2);
    assert_eq!(image.height, 2);
    assert_eq!(image.bits_per_component, 8);
    assert_eq!(image.color_space.as_deref(), Some("DeviceRGB"));
    assert_eq!(image.format, ImageFormat::Raw);
    assert_eq!(
        image.data,
        vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0]
    );
}

#[test]
fn jpeg_image_bytes_pass_through_unchanged() {
    let doc = Document::open("tests/fixtures/image_jpeg.pdf").expect("fixture should parse");
    let page = doc.page(0).expect("fixture has one page");

    let images = page.images().expect("image should be readable");
    assert_eq!(images.len(), 1);

    let image = &images[0];
    assert_eq!(image.format, ImageFormat::Jpeg);
    assert_eq!(image.data.first(), Some(&0xFF));
    assert_eq!(image.data.get(1), Some(&0xD8));
    assert_eq!(image.data.last(), Some(&0xD9));
    assert_eq!(image.data[image.data.len() - 2], 0xFF);
}

#[test]
fn unsupported_filter_is_reported_not_silently_dropped() {
    let doc = Document::open("tests/fixtures/image_unsupported_filter.pdf")
        .expect("fixture should parse");
    let page = doc.page(0).expect("fixture has one page");

    let err = page
        .images()
        .expect_err("CCITTFaxDecode should be unsupported");
    assert!(matches!(err, Error::Unsupported(_)));
}

#[test]
fn page_with_no_xobject_resources_extracts_no_images() {
    let doc = Document::open("tests/fixtures/minimal.pdf").expect("fixture should parse");
    let page = doc.page(0).expect("fixture has one page");

    assert_eq!(
        page.images().expect("page with no images should not error"),
        Vec::new()
    );
    assert_eq!(
        doc.images()
            .expect("document with no images should not error"),
        Vec::new()
    );
}

#[test]
fn document_images_matches_the_single_page_for_a_one_page_document() {
    let doc = Document::open("tests/fixtures/image_flate.pdf").expect("fixture should parse");
    let page = doc.page(0).expect("fixture has one page");

    assert_eq!(
        doc.images().expect("document should have images"),
        page.images().expect("page should have images"),
    );
}
