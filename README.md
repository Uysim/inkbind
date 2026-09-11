# inkbind

A helper library for reading, extracting from, and manipulating PDF documents, written in Rust.

> **Status: early development.** The crate ships its public API surface
> (`Document`, `Page`, `Metadata`, the error type) as documented stubs; every
> operation that requires real PDF parsing returns `Error::Unsupported` until
> its roadmap step lands. Feature work (document loading, text/metadata
> extraction, merge/split, and more) is tracked task-by-task.

## Layout

| Module            | Purpose                                            |
| ----------------- | ------------------------------------------------- |
| `inkbind::document` | `Document`, `Page` — load a PDF and walk its object model (parsing lands in Step 4). |
| `inkbind::text`     | Extract text from a page or document (Step 5).  |
| `inkbind::metadata` | `Metadata` — read document information (Step 6). |
| `inkbind::Error`    | The crate-wide error type.                       |

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
