# inkbind

A helper library for reading, extracting from, and manipulating PDF documents, written in Rust.

> **Status: pre-release.** Loading, text/metadata/image extraction,
> merging, splitting, rotating, reordering, watermarking, and building new
> documents from scratch are all implemented — see [Layout](#layout) below
> for exactly which module covers what. The crate has not yet been
> published to crates.io. Feature work is tracked task-by-task; see
> [`docs/adr/`](docs/adr/) for the design decisions behind it.

## Quick start

```rust
use inkbind::{Document, DocumentBuilder, PageSpec, TextLine};

// Build a one-page document from scratch...
let mut builder = DocumentBuilder::new();
builder.add_page(PageSpec::default().with_line(TextLine {
    text: "Hello, world!".to_owned(),
    x: 72.0,
    y: 720.0,
    font_size: 24.0,
}));
let doc = builder.build().unwrap();

// ...serialize it to bytes (or `doc.save("out.pdf")` to write a file)...
let bytes = doc.to_bytes().unwrap();

// ...and read it back like any other PDF.
let reopened = Document::from_bytes(&bytes).unwrap();
assert!(reopened.page(0).unwrap().text().unwrap().contains("Hello, world!"));
```

Opening an existing file works the same way, via `Document::open`:

```rust
use inkbind::Document;

let doc = Document::open("input.pdf").unwrap();
println!("{} pages", doc.page_count());
println!("{}", doc.text().unwrap());
println!("{:?}", doc.metadata().unwrap());
```

Every module below carries its own runnable example in its API docs
(`cargo doc --open`, or [docs.rs](https://docs.rs/inkbind) once published).

## Layout

| Module               | Purpose                                                                                          |
| -------------------- | ------------------------------------------------------------------------------------------------- |
| `inkbind::document`  | `Document`, `Page` — load a PDF (`open`/`from_bytes`), walk its pages, and serialize it back out (`to_bytes`/`save`). |
| `inkbind::text`      | Extract text from a page or a whole document.                                                     |
| `inkbind::metadata`  | `Metadata` — read title/author/subject/etc. from the `/Info` dictionary.                          |
| `inkbind::images`    | `Image`, `ImageFormat` — extract embedded raster images from a page's resources.                  |
| `inkbind::merge`     | Combine multiple documents into one, concatenating their pages in order.                          |
| `inkbind::split`     | Derive new documents from individual pages or arbitrary page ranges.                              |
| `inkbind::rotate`    | Rotate a single page by a multiple of 90 degrees.                                                 |
| `inkbind::reorder`   | Reorder a document's page sequence.                                                                |
| `inkbind::watermark` | `WatermarkOptions` — stamp text onto every page, with configurable size, rotation, and opacity.    |
| `inkbind::writer`    | `DocumentBuilder`, `PageSpec`, `TextLine` — build a new document from scratch.                     |
| `inkbind::Error`     | The crate-wide error type.                                                                         |

## Development

```sh
cargo test
cargo fmt --all --check
cargo clippy --all-targets --all-features
```

CI runs the same checks on `stable` and on the crate's MSRV (`1.75`).

## Design decisions

Architecture decision records live in [`docs/adr/`](docs/adr/). The parsing
backend is `lopdf`, wrapped behind inkbind's own types — see
[ADR 0001](docs/adr/0001-pdf-parsing-approach.md).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
