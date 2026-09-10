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

Used by `tests/parsing_approach_spike.rs` (ADR 0001).
