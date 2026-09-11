//! Step 12 — stamping text watermarks onto every page of a document.
//!
//! Verifies the watermark round-trips through inkbind's own, already-tested
//! `Page::text()`/`Document::images()` readers rather than asserting on raw
//! PDF structure: the watermark's content stream and font resource must be
//! well-formed enough for `lopdf`'s own extractor to read back, and the
//! resource-flattening the watermark performs must not disturb a page's
//! pre-existing text or embedded images.

use inkbind::{Document, Error, WatermarkOptions};

#[test]
fn watermark_text_is_readable_via_page_text() {
    let doc = Document::open("tests/fixtures/minimal.pdf").expect("fixture should parse");

    let watermarked = doc
        .watermark_text("DRAFT", WatermarkOptions::default())
        .expect("watermarking should succeed");

    let text = watermarked
        .page(0)
        .expect("page exists")
        .text()
        .expect("page should have text");
    assert!(
        text.contains("DRAFT"),
        "expected watermark text in page text, got {text:?}"
    );
}

#[test]
fn watermark_preserves_the_original_page_text() {
    let doc = Document::open("tests/fixtures/minimal.pdf").expect("fixture should parse");

    let watermarked = doc
        .watermark_text("DRAFT", WatermarkOptions::default())
        .expect("watermarking should succeed");

    let text = watermarked
        .page(0)
        .expect("page exists")
        .text()
        .expect("page should have text");
    assert!(
        text.contains("Hello Inkbind"),
        "expected original text to survive watermarking, got {text:?}"
    );
}

#[test]
fn watermark_applies_to_every_page() {
    let doc = Document::open("tests/fixtures/two_pages.pdf").expect("fixture should parse");

    let watermarked = doc
        .watermark_text("CONFIDENTIAL", WatermarkOptions::default())
        .expect("watermarking should succeed");

    assert_eq!(watermarked.page_count(), 2);
    for index in 0..2 {
        let text = watermarked
            .page(index)
            .expect("page exists")
            .text()
            .expect("page should have text");
        assert!(
            text.contains("CONFIDENTIAL"),
            "expected watermark on page {index}, got {text:?}"
        );
    }

    let first = watermarked.page(0).unwrap().text().unwrap();
    let second = watermarked.page(1).unwrap().text().unwrap();
    assert!(first.contains("Page One"));
    assert!(second.contains("Page Two"));
}

#[test]
fn watermark_does_not_change_page_count() {
    let doc = Document::open("tests/fixtures/two_pages.pdf").expect("fixture should parse");

    let watermarked = doc
        .watermark_text("DRAFT", WatermarkOptions::default())
        .expect("watermarking should succeed");

    assert_eq!(watermarked.page_count(), doc.page_count());
}

#[test]
fn watermark_preserves_existing_embedded_images() {
    let doc = Document::open("tests/fixtures/image_flate.pdf").expect("fixture should parse");
    let before = doc.images().expect("fixture should have a decodable image");
    assert_eq!(before.len(), 1);

    let watermarked = doc
        .watermark_text("DRAFT", WatermarkOptions::default())
        .expect("watermarking should succeed");
    let after = watermarked
        .images()
        .expect("watermarked document should still decode its image");

    assert_eq!(before, after);
}

#[test]
fn watermark_rejects_opacity_above_one() {
    let doc = Document::open("tests/fixtures/minimal.pdf").expect("fixture should parse");
    let options = WatermarkOptions {
        opacity: 1.5,
        ..WatermarkOptions::default()
    };

    let err = doc
        .watermark_text("DRAFT", options)
        .expect_err("opacity above 1.0 should be rejected");
    assert!(matches!(err, Error::InvalidOpacity(v) if v == 1.5));
}

#[test]
fn watermark_rejects_negative_opacity() {
    let doc = Document::open("tests/fixtures/minimal.pdf").expect("fixture should parse");
    let options = WatermarkOptions {
        opacity: -0.1,
        ..WatermarkOptions::default()
    };

    let err = doc
        .watermark_text("DRAFT", options)
        .expect_err("negative opacity should be rejected");
    assert!(matches!(err, Error::InvalidOpacity(v) if v == -0.1));
}

#[test]
fn watermark_options_default_values() {
    let options = WatermarkOptions::default();
    assert_eq!(options.font_size, 48.0);
    assert_eq!(options.rotation, 45.0);
    assert_eq!(options.opacity, 0.3);
}
