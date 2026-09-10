# ADR 0001 — Base PDF parsing approach

- **Status:** Accepted (2026-09-10)
- **Task:** Step 2 — "Evaluate base PDF parsing approach"
- **Deciders:** inkbind maintainers
- **Supersedes / superseded by:** —

## Context

inkbind needs to turn PDF bytes into an object model it can read from and write
back. The roadmap that depends on this choice:

| Step | Needs from the parsing layer |
|------|------------------------------|
| 4 — document loading & object model | header/xref parsing, indirect-object resolution, page tree |
| 5 — text extraction | decoded content streams, font/encoding dictionaries |
| 6 — metadata extraction | trailer `/Info` dict, XMP stream |
| 9 — merge | parse N docs, renumber objects, re-serialize |
| 10 — split | page-tree surgery, re-serialize |
| 11 — rotate / reorder | mutate page attributes, re-serialize |
| 12 — watermark / stamp | append content streams + resources |
| 13 — basic writer | serialize a document built in memory |

So we need **both** a reader and a writer over a shared mutable object model —
not just an extraction-only reader.

Hard constraint: the crate's MSRV is **Rust 1.75** (`rust-version = "1.75"`,
enforced in CI). Any dependency must build on 1.75.

## Options considered

### A. `lopdf` — low-level object model (reader + writer)

- Models the PDF COS layer directly: `Document`, `Object`, `Dictionary`,
  `Stream`, `ObjectId`. Parses classic xref tables and xref streams, resolves
  indirect references, decodes the common stream filters (Flate, LZW, ASCII85,
  ASCIIHex, RunLength), walks the page tree, and **serializes back to bytes**
  (`save`, `save_to`), including object renumbering helpers used for merge.
- Pure Rust with the `nom_parser` feature; no C toolchain. MIT licensed.
- Used as the foundation by other ecosystem crates (e.g. `printpdf`, `genpdf`).
- Cons: the API is deliberately low-level — text extraction and font handling
  are left to the caller; the API churns across minor versions; the current
  release (0.45) and everything back to 0.34 pull dependencies that require
  `edition2024` (see "MSRV findings").

### B. `pdf` (a.k.a. `pdf-rs`) — typed, read-oriented

- Higher-level typed views (`File`, `Page`, `Resources`) with lazy loading;
  pleasant for reading and extraction.
- Writing / mutation / re-serialization is not a first-class capability, which
  blocks Steps 9–13. We would still need a second crate (or hand-rolled writer)
  for the manipulation half of inkbind.

### C. `printpdf` — generation only

- A *writer*, not a parser (and itself built on `lopdf`). Cannot load an
  existing document. Only relevant later, as one option for Step 13.

### D. From-scratch parser

- Full control of the object model and MSRV; zero dependency risk.
- But a conformant reader is a large surface: xref tables **and** xref streams,
  object streams, the filter zoo, linearization, and (for real-world files)
  encryption. This is months of work and a long bug tail for a helper library,
  with no differentiation — parsing is not where inkbind adds value.

## MSRV findings (measured 2026-09-10)

Reproduced with `cargo +1.75.0`:

| lopdf | Builds on Rust 1.75? | Notes |
|-------|----------------------|-------|
| 0.45 (latest) | ❌ | transitively requires `edition2024` |
| 0.36 | ❌ | no `nom_parser` feature; deps need `edition2024` |
| 0.34 | ❌ | pulls `hashbrown 0.17` → `edition2024` |
| **0.32** | ✅ | with a small set of `Cargo.lock` pins (below) |

`lopdf = "=0.32.0"` with `default-features = false, features = ["nom_parser"]`
builds and runs on both `stable` and `1.75.0` once these transitive crates are
pinned in `Cargo.lock` to their last 1.75-compatible releases:

```
time-core   0.1.2      encoding_rs  0.8.35
time-macros 0.2.18     flate2       1.0.35
time        0.3.36     miniz_oxide  0.8.9
deranged    0.3.11     num-conv     0.1.0
```

`Cargo.lock` is therefore now committed (it was `.gitignore`d while the crate
had no dependencies). Registry dependencies build with `--cap-lints allow`, so
these older versions do not trip the `-D warnings` CI gate on newer stable
toolchains.

## Decision

**Adopt `lopdf` as inkbind's low-level PDF parsing and object-model backend.**

1. Pin `lopdf = "=0.32.0"` (features: `nom_parser`, no defaults) and commit
   `Cargo.lock` with the pins listed above. Keep MSRV at 1.75.
2. **Wrap it.** `lopdf` types must not appear in inkbind's public API. inkbind
   exposes its own `Document`, `Page`, `Metadata`, … and keeps the `lopdf`
   handle private, so the backend can be swapped or supplemented later without a
   breaking change.
3. Own the higher layers ourselves: text extraction (Step 5) and any writer
   ergonomics (Step 13) are built on top of `lopdf`'s object model, not
   delegated to another crate.
4. At this step `lopdf` is a **dev-dependency** backing the evaluation spike
   only. **Step 4 promotes it to a normal dependency** as the object model lands.

### Validation

`tests/parsing_approach_spike.rs` exercises `lopdf` against
`tests/fixtures/minimal.pdf` (a 738-byte hand-built one-page PDF; see
`tests/fixtures/README.md`). It asserts the five capabilities the roadmap needs
— version parse, page-tree walk, `/Info` metadata read, content-stream decode,
and byte round-trip (save + reparse) — and runs green on `stable` and `1.75.0`.

## Consequences

**Positive**

- One crate covers both the read and write halves of the roadmap.
- Pure-Rust build; no C dependency; MIT license is compatible with
  `MIT OR Apache-2.0`.
- The wrapper boundary keeps the decision reversible.

**Negative / risks**

- Pinned to an old `lopdf` (0.32; latest is 0.45). We forgo newer fixes/features
  until we either raise the MSRV or backport. Mitigation: the wrapper isolates
  the version; revisit when a concrete need appears.
- `Cargo.lock` now carries hand-picked pins. A `cargo update` could break the
  MSRV build. Mitigation: the MSRV CI leg (`cargo +1.75.0`) already guards this;
  the spike test is the canary.
- `lopdf` gives us no text-extraction help — expected, and it is inkbind's job.

## Follow-ups

- **Step 4:** move `lopdf` to `[dependencies]`; introduce `inkbind::Document`
  wrapping `lopdf::Document`; decide error mapping into `inkbind::Error`.
- Track a future **"raise MSRV to track lopdf latest"** decision once a needed
  feature or fix lands only in a newer release.
- Consider a scheduled `cargo update --dry-run` / `cargo minimal-versions` check
  so pin drift is noticed early.
