//! Behavior tests for `Document::metadata` (Step 6).
//!
//! `tests/api_surface.rs` pins the method's *shape*; these tests pin its
//! actual `/Info` dictionary extraction behavior: full field reads, partial
//! (some fields absent), BOM-based UTF-16BE decoding, and the no-`/Info`
//! case.

use inkbind::{Document, Metadata};

#[test]
fn reads_every_info_field_when_all_are_present() {
    let doc = Document::open("tests/fixtures/full_metadata.pdf").expect("fixture should parse");

    let meta = doc.metadata().expect("metadata should be readable");

    assert_eq!(meta.title.as_deref(), Some("Inkbind Full Metadata Fixture"));
    assert_eq!(meta.author.as_deref(), Some("Inkbind Test Suite"));
    assert_eq!(
        meta.subject.as_deref(),
        Some("Exercising every /Info field")
    );
    assert_eq!(meta.keywords.as_deref(), Some("inkbind,pdf,metadata"));
    assert_eq!(meta.creator.as_deref(), Some("inkbind"));
    assert_eq!(
        meta.producer.as_deref(),
        Some("inkbind gen_full_metadata_pdf.py")
    );
    assert_eq!(
        meta.creation_date.as_deref(),
        Some("D:20240115093000+00'00'")
    );
}

#[test]
fn absent_info_fields_are_none() {
    // `minimal.pdf`'s /Info only populates /Title, /Producer, /Creator.
    let doc = Document::open("tests/fixtures/minimal.pdf").expect("fixture should parse");

    let meta = doc.metadata().expect("metadata should be readable");

    assert_eq!(meta.title.as_deref(), Some("Inkbind Minimal Fixture"));
    assert_eq!(meta.creator.as_deref(), Some("inkbind"));
    assert_eq!(meta.producer.as_deref(), Some("inkbind gen_minimal_pdf.py"));
    assert_eq!(meta.author, None);
    assert_eq!(meta.subject, None);
    assert_eq!(meta.keywords, None);
    assert_eq!(meta.creation_date, None);
}

#[test]
fn decodes_utf16be_title_with_bom() {
    let doc = Document::open("tests/fixtures/unicode_metadata.pdf").expect("fixture should parse");

    let meta = doc.metadata().expect("metadata should be readable");

    assert_eq!(meta.title.as_deref(), Some("café"));
    assert_eq!(
        meta.producer.as_deref(),
        Some("inkbind gen_unicode_metadata_pdf.py")
    );
}

#[test]
fn missing_info_dictionary_yields_default_metadata() {
    let doc = Document::open("tests/fixtures/no_info.pdf").expect("fixture should parse");

    let meta = doc
        .metadata()
        .expect("missing /Info should not be an error");

    assert_eq!(meta, Metadata::default());
}
