# Test fixtures

Tiny, hand-verifiable inputs for inkbind's test suite. Keep them minimal and
regenerable.

## `minimal.pdf` (738 bytes)

A single-page PDF 1.7 file with:

- Catalog → Pages → one `/MediaBox [0 0 612 792]` page
- an **uncompressed** content stream drawing the text `Hello Inkbind`
- a Type1 `/Helvetica` font resource
- an `/Info` dictionary: `/Title (Inkbind Minimal Fixture)`,
  `/Producer (inkbind gen_minimal_pdf.py)`, `/Creator (inkbind)`
- a classic (non-stream) cross-reference table + trailer

Because the content stream is stored uncompressed and the file uses a plain xref
table, every byte is readable in a text editor.

### Regenerate

```sh
python3 tests/fixtures/gen_minimal_pdf.py tests/fixtures/minimal.pdf
```

`gen_minimal_pdf.py` uses only the Python standard library. It computes the xref
byte offsets from the serialized output, so the file stays valid without manual
offset bookkeeping. Regeneration is deterministic — the bytes do not change
between runs.

Used by `tests/parsing_approach_spike.rs` (ADR 0001) and `tests/document_loading.rs`
(Step 4).

## `two_pages.pdf` (996 bytes)

A two-page PDF 1.7 file, structured like `minimal.pdf` but with:

- Catalog → Pages → two `/MediaBox [0 0 612 792]` pages, in document order
- distinct **uncompressed** content streams: page 1 draws `Page One`, page 2
  draws `Page Two`
- a shared Type1 `/Helvetica` font resource
- an `/Info` dictionary: `/Title (Inkbind Two-Page Fixture)`,
  `/Producer (inkbind gen_two_page_pdf.py)`, `/Creator (inkbind)`
- a classic (non-stream) cross-reference table + trailer

### Regenerate

```sh
python3 tests/fixtures/gen_two_page_pdf.py tests/fixtures/two_pages.pdf
```

Used by `tests/document_loading.rs` (Step 4) to check that page enumeration
and indexing hold across more than one page. Its distinct per-page text
(`Page One` / `Page Two`) also makes it useful for `tests/text_extraction.rs`
(Step 5): it pins ordering across pages.

## `no_text.pdf` (592 bytes)

A single-page PDF 1.7 file with:

- Catalog → Pages → one `/MediaBox [0 0 612 792]` page
- a **zero-length** content stream — no `BT`/`ET`, no text operators at all
- an empty `/Font` resource dict, kept for shape-parity with the other
  fixtures even though no font is ever selected
- an `/Info` dictionary: `/Title (Inkbind No-Text Fixture)`,
  `/Producer (inkbind gen_no_text_pdf.py)`, `/Creator (inkbind)`
- a classic (non-stream) cross-reference table + trailer

### Regenerate

```sh
python3 tests/fixtures/gen_no_text_pdf.py tests/fixtures/no_text.pdf
```

Used by `tests/text_extraction.rs` (Step 5) to pin that a page with no text
content extracts as `Ok("")`, not an error.

## `multiline.pdf` (782 bytes)

A single-page PDF 1.7 file with:

- Catalog → Pages → one `/MediaBox [0 0 612 792]` page
- an **uncompressed** content stream with **two separate** `BT ... ET`
  blocks at different `Td` offsets, drawing `First Line` then `Second Line`
- a Type1 `/Helvetica` font resource
- an `/Info` dictionary: `/Title (Inkbind Multiline Fixture)`,
  `/Producer (inkbind gen_multiline_pdf.py)`, `/Creator (inkbind)`
- a classic (non-stream) cross-reference table + trailer

### Regenerate

```sh
python3 tests/fixtures/gen_multiline_pdf.py tests/fixtures/multiline.pdf
```

Used by `tests/text_extraction.rs` (Step 5) to pin ordering/line-separation
*within* a single page's extracted text (`two_pages.pdf` only covers
ordering *across* pages).
