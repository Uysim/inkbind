//! Step 14 — benchmarks for building, serializing, parsing, and extracting
//! from a large (multi-hundred-page) document, to identify bottlenecks.
//!
//! Run `cargo bench` for full statistical sampling. `cargo test --all-targets`
//! (which includes `--benches`) instead runs each benchmarked closure once,
//! via criterion's built-in detection of cargo's `--test` flag — that's a
//! smoke check that the harness compiles and nothing panics, not a
//! performance measurement, and is what CI relies on.
//!
//! `missing_docs` is silenced here because `criterion_group!` expands to an
//! undocumented function; this bench binary isn't part of the crate's
//! public API, so the lint doesn't apply.
#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use inkbind::{Document, DocumentBuilder, PageSpec, TextLine};

const PAGE_COUNT: usize = 300;
const LINES_PER_PAGE: usize = 10;

fn build_large_document() -> Document {
    let mut builder = DocumentBuilder::new();
    for page_index in 0..PAGE_COUNT {
        let mut page = PageSpec::default();
        for line_index in 0..LINES_PER_PAGE {
            page = page.with_line(TextLine {
                text: format!("Page {page_index} line {line_index}"),
                x: 72.0,
                y: 720.0 - (line_index as f32 * 20.0),
                font_size: 12.0,
            });
        }
        builder.add_page(page);
    }
    builder.build().expect("large document should build")
}

fn bench_build(c: &mut Criterion) {
    c.bench_function("build_large_document", |b| b.iter(build_large_document));
}

fn bench_serialize(c: &mut Criterion) {
    let doc = build_large_document();
    c.bench_function("serialize_large_document", |b| {
        b.iter(|| doc.to_bytes().expect("serialization should succeed"));
    });
}

fn bench_parse(c: &mut Criterion) {
    let bytes = build_large_document()
        .to_bytes()
        .expect("serialization should succeed");
    c.bench_function("parse_large_document", |b| {
        b.iter(|| Document::from_bytes(&bytes).expect("parsing should succeed"));
    });
}

fn bench_text_extraction(c: &mut Criterion) {
    let bytes = build_large_document()
        .to_bytes()
        .expect("serialization should succeed");
    let doc = Document::from_bytes(&bytes).expect("parsing should succeed");
    c.bench_function("extract_text_from_large_document", |b| {
        b.iter(|| doc.text().expect("text extraction should succeed"));
    });
}

fn bench_metadata_extraction(c: &mut Criterion) {
    let bytes = build_large_document()
        .to_bytes()
        .expect("serialization should succeed");
    let doc = Document::from_bytes(&bytes).expect("parsing should succeed");
    c.bench_function("extract_metadata_from_large_document", |b| {
        b.iter(|| doc.metadata().expect("metadata extraction should succeed"));
    });
}

criterion_group!(
    benches,
    bench_build,
    bench_serialize,
    bench_parse,
    bench_text_extraction,
    bench_metadata_extraction
);
criterion_main!(benches);
