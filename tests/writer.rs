//! Step 13 — building new PDF documents from scratch, and serializing any
//! document (generated or loaded) back to bytes/a file.
//!
//! Verifies round-trips through inkbind's own, already-tested readers
//! (`Document::from_bytes`/`Document::open`, `Page::text`) rather than
//! asserting on raw PDF structure — same philosophy as `tests/watermark.rs`.

use inkbind::{Document, DocumentBuilder, PageSpec, TextLine};

fn line(text: &str, x: f32, y: f32) -> TextLine {
    TextLine {
        text: text.to_owned(),
        x,
        y,
        font_size: 24.0,
    }
}

#[test]
fn built_single_page_document_round_trips_its_text() {
    let mut builder = DocumentBuilder::new();
    builder.add_page(PageSpec::default().with_line(line("Hello, world!", 72.0, 720.0)));
    let doc = builder.build().expect("build should succeed");

    let bytes = doc.to_bytes().expect("serialization should succeed");
    let reopened = Document::from_bytes(&bytes).expect("reparsing should succeed");

    assert_eq!(reopened.page_count(), 1);
    let text = reopened.page(0).unwrap().text().unwrap();
    assert!(
        text.contains("Hello, world!"),
        "expected generated text, got {text:?}"
    );
}

#[test]
fn multiple_lines_on_one_page_all_appear_in_its_text() {
    let mut builder = DocumentBuilder::new();
    builder.add_page(
        PageSpec::default()
            .with_line(line("First line", 72.0, 700.0))
            .with_line(line("Second line", 72.0, 650.0)),
    );
    let doc = builder.build().expect("build should succeed");

    let bytes = doc.to_bytes().expect("serialization should succeed");
    let reopened = Document::from_bytes(&bytes).expect("reparsing should succeed");

    let text = reopened.page(0).unwrap().text().unwrap();
    assert!(text.contains("First line"), "got {text:?}");
    assert!(text.contains("Second line"), "got {text:?}");
}

#[test]
fn multiple_pages_keep_independent_text_in_order() {
    let mut builder = DocumentBuilder::new();
    builder
        .add_page(PageSpec::default().with_line(line("Page One", 72.0, 720.0)))
        .add_page(PageSpec::default().with_line(line("Page Two", 72.0, 720.0)));
    let doc = builder.build().expect("build should succeed");

    let bytes = doc.to_bytes().expect("serialization should succeed");
    let reopened = Document::from_bytes(&bytes).expect("reparsing should succeed");

    assert_eq!(reopened.page_count(), 2);
    let first = reopened.page(0).unwrap().text().unwrap();
    let second = reopened.page(1).unwrap().text().unwrap();
    assert!(first.contains("Page One") && !first.contains("Page Two"));
    assert!(second.contains("Page Two") && !second.contains("Page One"));
}

#[test]
fn builder_with_no_pages_produces_a_valid_zero_page_document() {
    let doc = DocumentBuilder::new()
        .build()
        .expect("build should succeed");
    assert_eq!(doc.page_count(), 0);

    let bytes = doc.to_bytes().expect("serialization should succeed");
    let reopened = Document::from_bytes(&bytes).expect("reparsing should succeed");
    assert_eq!(reopened.page_count(), 0);
}

#[test]
fn save_to_a_file_path_round_trips_via_document_open() {
    let mut builder = DocumentBuilder::new();
    builder.add_page(PageSpec::default().with_line(line("Saved to disk", 72.0, 720.0)));
    let doc = builder.build().expect("build should succeed");

    let path = std::env::temp_dir().join(format!("inkbind-writer-test-{}.pdf", std::process::id()));
    doc.save(&path).expect("save should succeed");

    let reopened = Document::open(&path).expect("reopening the saved file should succeed");
    let text = reopened.page(0).unwrap().text().unwrap();
    assert!(text.contains("Saved to disk"), "got {text:?}");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn rotated_fixture_survives_a_serialize_reparse_round_trip() {
    let doc = Document::open("tests/fixtures/two_pages.pdf").expect("fixture should parse");
    let rotated = doc.rotate_page(1, 90).expect("rotate should succeed");

    let bytes = rotated.to_bytes().expect("serialization should succeed");
    let reopened = Document::from_bytes(&bytes).expect("reparsing should succeed");

    assert_eq!(reopened.page_count(), 2);
    assert_eq!(reopened.page(0).unwrap().rotation(), 0);
    assert_eq!(reopened.page(1).unwrap().rotation(), 90);
    assert!(reopened
        .page(1)
        .unwrap()
        .text()
        .unwrap()
        .contains("Page Two"));
}
