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

## `full_metadata.pdf` (892 bytes)

A single-page PDF 1.7 file with:

- Catalog → Pages → one `/MediaBox [0 0 612 792]` page
- an **uncompressed** content stream drawing the text `Full Metadata`
- a Type1 `/Helvetica` font resource
- an `/Info` dictionary with **all seven** fields `inkbind` reads:
  `/Title`, `/Author`, `/Subject`, `/Keywords`, `/Creator`, `/Producer`,
  `/CreationDate` (`D:20240115093000+00'00'`) — plain ASCII literal strings
- a classic (non-stream) cross-reference table + trailer

### Regenerate

```sh
python3 tests/fixtures/gen_full_metadata_pdf.py tests/fixtures/full_metadata.pdf
```

Used by `tests/metadata_extraction.rs` (Step 6) to pin that every `/Info`
field reaches `Metadata` when present.

## `unicode_metadata.pdf` (718 bytes)

A single-page PDF 1.7 file with:

- Catalog → Pages → one `/MediaBox [0 0 612 792]` page
- an **uncompressed** content stream drawing the text `Unicode Metadata`
- a Type1 `/Helvetica` font resource
- an `/Info` dictionary whose `/Title` is the PDF string literal
  `(<0xFE 0xFF BOM><UTF-16BE "café">)` — the encoding real PDF producers
  use for non-ASCII `/Info` text strings — plus a plain-ASCII `/Producer`
  for shape-parity with the other fixtures
- a classic (non-stream) cross-reference table + trailer

### Regenerate

```sh
python3 tests/fixtures/gen_unicode_metadata_pdf.py tests/fixtures/unicode_metadata.pdf
```

Used by `tests/metadata_extraction.rs` (Step 6) to pin BOM-based UTF-16BE
decoding of `/Info` text strings.

## `no_info.pdf` (588 bytes)

A single-page PDF 1.7 file with:

- Catalog → Pages → one `/MediaBox [0 0 612 792]` page
- an **uncompressed** content stream drawing the text `No Info`
- a Type1 `/Helvetica` font resource
- a trailer with **no `/Info` entry at all** — the `/Info` dictionary is
  optional per the PDF spec
- a classic (non-stream) cross-reference table + trailer

### Regenerate

```sh
python3 tests/fixtures/gen_no_info_pdf.py tests/fixtures/no_info.pdf
```

Used by `tests/metadata_extraction.rs` (Step 6) to pin that a missing
`/Info` dictionary yields `Ok(Metadata::default())`, not an error.

## `compressed_content.pdf` (793 bytes)

A single-page PDF 1.7 file with:

- Catalog → Pages → one `/MediaBox [0 0 612 792]` page
- a content stream drawing the text `Compressed Content`, stored
  **zlib-compressed** with `/Filter /FlateDecode` — every fixture above
  stores its content stream uncompressed
- a Type1 `/Helvetica` font resource
- an `/Info` dictionary: `/Title (Inkbind Compressed Content Fixture)`,
  `/Producer (inkbind gen_compressed_content_pdf.py)`, `/Creator (inkbind)`
- a classic (non-stream) cross-reference table + trailer

### Regenerate

```sh
python3 tests/fixtures/gen_compressed_content_pdf.py tests/fixtures/compressed_content.pdf
```

Used by `tests/sample_pdf_corpus.rs` (Step 7) to pin that text extraction
transparently decompresses a `FlateDecode` content stream — real PDF
producers compress content streams almost universally, unlike every
hand-crafted fixture used through Step 6.

## `multi_stream_contents.pdf` (894 bytes)

A single-page PDF 1.7 file with:

- Catalog → Pages → one `/MediaBox [0 0 612 792]` page
- `/Contents [4 0 R 5 0 R]` — the page's content is **two separate,
  uncompressed content-stream objects** (not one stream with multiple
  `BT`/`ET` blocks, which `multiline.pdf` already covers), drawing
  `Array Stream One` and `Array Stream Two` respectively
- a shared Type1 `/Helvetica` font resource
- an `/Info` dictionary: `/Title (Inkbind Multi-Stream Contents Fixture)`,
  `/Producer (inkbind gen_multi_stream_contents_pdf.py)`, `/Creator (inkbind)`
- a classic (non-stream) cross-reference table + trailer

### Regenerate

```sh
python3 tests/fixtures/gen_multi_stream_contents_pdf.py tests/fixtures/multi_stream_contents.pdf
```

Used by `tests/sample_pdf_corpus.rs` (Step 7) to pin that text extraction
concatenates content across distinct stream *objects* referenced by a
`/Contents` array, not just multiple text blocks within one stream.

## `xref_stream.pdf` (710 bytes)

A single-page PDF 1.5 file that uses a **cross-reference stream**
(`/Type /XRef`) instead of a classic `xref` table + `trailer` keyword —
the modern alternative most current PDF producers emit, and a
structurally different parse path from every other fixture in this suite:

- `startxref` points directly at an uncompressed
  `<< /Type /XRef /W [1 4 2] /Size 8 /Root 1 0 R /Info 6 0 R >>` stream
  object whose binary rows encode each object's type/offset/generation —
  no separate `xref`/`trailer` keywords appear anywhere in the file
- Catalog → Pages → one `/MediaBox [0 0 612 792]` page, drawing the text
  `Xref Stream`, otherwise identical in shape to `minimal.pdf` — only the
  cross-reference mechanism changes, isolating that one variable
- a Type1 `/Helvetica` font resource
- an `/Info` dictionary: `/Title (Inkbind Xref Stream Fixture)`,
  `/Producer (inkbind gen_xref_stream_pdf.py)`, `/Creator (inkbind)`

### Regenerate

```sh
python3 tests/fixtures/gen_xref_stream_pdf.py tests/fixtures/xref_stream.pdf
```

Used by `tests/sample_pdf_corpus.rs` (Step 7) to pin that document loading,
text extraction, and metadata reading all work when a PDF's
cross-references live in a stream object instead of a classic xref table.

## `image_flate.pdf` (844 bytes)

A single-page PDF 1.7 file with:

- Catalog → Pages → one `/MediaBox [0 0 612 792]` page
- a content stream that paints the image (`/Im0 Do`) — not exercised by
  inkbind's image extraction, included only so the file is a well-formed,
  renderable PDF
- one Image XObject (`/Im0`): 2x2 pixels, 8-bit `/DeviceRGB`, raw pixel
  data `(255,0,0) (0,255,0) / (0,0,255) (255,255,0)` (row-major, no
  padding) **zlib-compressed** with `/Filter /FlateDecode` — the same
  filter real-world PDF producers commonly use for lossless embedded
  images
- an `/Info` dictionary: `/Title (Inkbind Image Flate Fixture)`,
  `/Producer (inkbind gen_image_flate_pdf.py)`, `/Creator (inkbind)`
- a classic (non-stream) cross-reference table + trailer

### Regenerate

```sh
python3 tests/fixtures/gen_image_flate_pdf.py tests/fixtures/image_flate.pdf
```

Used by `tests/image_extraction.rs` (Step 8) to pin that image extraction
decompresses a `/FlateDecode` Image XObject to its exact raw pixel bytes.

## `image_jpeg.pdf` (865 bytes)

A single-page PDF 1.7 file with:

- Catalog → Pages → one `/MediaBox [0 0 612 792]` page
- a content stream that paints the image (`/Im0 Do`), as in
  `image_flate.pdf`
- one Image XObject (`/Im0`) with `/Filter /DCTDecode`, whose stream bytes
  are a **placeholder** (SOI/EOI JPEG markers wrapping a fixed string) —
  **not** a real, decodable JPEG. inkbind never decodes JPEG pixel data
  itself; it only passes `/DCTDecode` stream bytes through unchanged, so a
  placeholder is sufficient to test that routing.
- an `/Info` dictionary: `/Title (Inkbind Image JPEG Fixture)`,
  `/Producer (inkbind gen_image_jpeg_pdf.py)`, `/Creator (inkbind)`
- a classic (non-stream) cross-reference table + trailer

### Regenerate

```sh
python3 tests/fixtures/gen_image_jpeg_pdf.py tests/fixtures/image_jpeg.pdf
```

Used by `tests/image_extraction.rs` (Step 8) to pin that a `/DCTDecode`
Image XObject's bytes pass through unchanged as `ImageFormat::Jpeg`.

## `image_unsupported_filter.pdf` (901 bytes)

A single-page PDF 1.7 file with:

- Catalog → Pages → one `/MediaBox [0 0 612 792]` page
- a content stream that paints the image (`/Im0 Do`), as in
  `image_flate.pdf`
- one Image XObject (`/Im0`) with `/Filter /CCITTFaxDecode` and arbitrary
  placeholder bytes — inkbind does not decode this filter, so the exact
  bytes are irrelevant to the test
- an `/Info` dictionary: `/Title (Inkbind Image Unsupported Filter
  Fixture)`, `/Producer (inkbind gen_image_unsupported_filter_pdf.py)`,
  `/Creator (inkbind)`
- a classic (non-stream) cross-reference table + trailer

### Regenerate

```sh
python3 tests/fixtures/gen_image_unsupported_filter_pdf.py tests/fixtures/image_unsupported_filter.pdf
```

Used by `tests/image_extraction.rs` (Step 8) to pin that an Image XObject
using a filter inkbind does not decode surfaces `Error::Unsupported`
rather than being silently dropped.
