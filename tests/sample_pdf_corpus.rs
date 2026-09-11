//! Step 7 — a small corpus of structurally distinct sample PDFs, each
//! exercising a real-world variation the fixtures used in Steps 4-6 never
//! touched: a compressed content stream, a PDF 1.5+ cross-reference stream
//! (instead of a classic xref table), and a page whose `/Contents` is an
//! array of multiple stream objects. Each test below exercises document
//! loading, text extraction, and metadata reading together against one
//! fixture, rather than pinning a single dimension in isolation like
//! `document_loading.rs`/`text_extraction.rs`/`metadata_extraction.rs` do.

use inkbind::Document;

#[test]
fn flate_decode_compressed_content_stream_is_transparently_decoded() {
    let doc =
        Document::open("tests/fixtures/compressed_content.pdf").expect("fixture should parse");

    assert_eq!(doc.page_count(), 1);
    assert_eq!(
        doc.text().expect("compressed stream should decode"),
        "Compressed Content\n"
    );
    assert_eq!(
        doc.metadata()
            .expect("metadata should be readable")
            .title
            .as_deref(),
        Some("Inkbind Compressed Content Fixture")
    );
}

#[test]
fn page_contents_array_of_multiple_streams_is_concatenated_in_order() {
    let doc =
        Document::open("tests/fixtures/multi_stream_contents.pdf").expect("fixture should parse");
    let page = doc.page(0).expect("fixture has one page");

    assert_eq!(doc.page_count(), 1);
    assert_eq!(
        page.text().expect("both content streams should decode"),
        "Array Stream One\nArray Stream Two\n"
    );
    assert_eq!(
        doc.metadata()
            .expect("metadata should be readable")
            .title
            .as_deref(),
        Some("Inkbind Multi-Stream Contents Fixture")
    );
}

#[test]
fn cross_reference_stream_is_parsed_like_a_classic_xref_table() {
    let doc = Document::open("tests/fixtures/xref_stream.pdf").expect("fixture should parse");

    assert_eq!(doc.page_count(), 1);
    assert_eq!(
        doc.text().expect("document should have text"),
        "Xref Stream\n"
    );
    assert_eq!(
        doc.metadata()
            .expect("metadata should be readable")
            .title
            .as_deref(),
        Some("Inkbind Xref Stream Fixture")
    );
}
