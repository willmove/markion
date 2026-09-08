# DOCX import compatibility evidence

This file separates automated semantic evidence from real-producer and native
platform checks. It was last updated on 2026-09-08.

## Reader selection

The candidate gate evaluated released `docx-rs` 0.4.22 (MIT), with image
previews disabled. `cargo info docx-rs` and a disposable headless harness were
used against a package containing body text, insertion/deletion, move-to and
move-from revisions, a footnote, OMML, and `altChunk`. Serialization of the
parsed model produced these observations:

| Probe | Present in parsed model |
| --- | --- |
| Body text | yes |
| Inserted text | yes |
| Deleted text | yes |
| Move-to text | yes |
| Move-from text | yes |
| Footnote definition text | no |
| OMML token | no |
| `altChunk` token/inventory | no |

Source inspection also found that `read_zip` preallocates from the declared ZIP
entry size and then calls unbounded `read_to_end`; its public reader has no
cancellation, deadline, decompressed-byte, entry-count, XML-depth, or output
budget. Many element readers discard unknown branches through `_ => {}`. The
model exposes styles, numbering, relationships, comments and some revision
nodes, but adopting it would require a second package inventory and still
would not preserve enough data to implement the required loss report and OMML
contract. Peak memory was not measured because the candidate failed the
mandatory bounded-read and visibility gates before corpus qualification.

The selected implementation is therefore the focused reader in
`markion-docx-import`:

| Dependency | Version/features | License | Purpose |
| --- | --- | --- | --- |
| `zip` | 0.6.6, `deflate` only | MIT | bounded package entry consumption |
| `quick-xml` | 0.39.4, no features | MIT | streaming XML validation, depth/deadline/cancellation checkpoints |
| `roxmltree` | 0.21.1, defaults | MIT OR Apache-2.0 | namespace-aware traversal after bounded reads |

The runtime contains no `docx-rs`, Pandoc, JS, browser, or office-suite import
dependency. The rejected harness lived outside the repository and no rejected
reader code or dependency was integrated.

## Automated fixtures

`cargo test -p markion-docx-import` currently runs 17 deterministic package
fixtures. They cover:

- multilingual styled text, Markdown delimiter escaping, explicit line breaks,
  inherited paragraph/character styles, and accepted inline revisions;
- numbered-list start overrides, safe/unsafe hyperlinks, and Unicode bookmarks;
- headerless GFM tables plus HTML `rowspan`/`colspan` merged tables;
- repeated footnote references and multi-paragraph definitions;
- embedded image validation, byte deduplication, typed asset references, alt
  text, external-image blocking, and visible fallbacks;
- supported inline fraction/radical and display-script OMML with exact LaTeX
  expectations validated by Markion's existing `MathRenderer`, plus diagnosed
  visible fallback text for unsupported OMML;
- deleted paragraph-mark joining, move-range ambiguity, unsupported-content
  inventory, cached field results, and placeholder-only rejection;
- source/decompressed/XML/entry/depth/deadline/cancellation limits, entities,
  escaping relationships, external main documents, duplicate parts, macro and
  encrypted packages.

Root persistence tests cover typed URL materialization, Markdown-like authored
placeholder text, Chinese and space-containing paths, create-only collisions,
assets-first publication, repeated byte reuse, and text-only imports without an
empty asset directory. App tests cover localized report severity and excerpts.

These are synthetic semantic fixtures. They do not substitute for the required
real-producer corpus.

## Real-producer corpus

No redistributable Microsoft Word, WPS Office, or LibreOffice corpus was
available in this workspace on 2026-09-08. Native Markion and MarkNice exports
also have not yet been captured as durable corpus files. Consequently producer
compatibility, required 20–30-file coverage, and comparative output claims are
**pending**. Each future fixture must record:

- producer name, exact version/platform, creation steps and redistribution or
  sanitization provenance;
- expected accepted-view text and order, headings/lists/tables/math/notes,
  asset hashes and expected diagnostics;
- import result and elapsed/peak-memory measurements, with any exception
  reviewed as an explicit scope decision rather than accepted as parser output.

## Performance and native platform evidence

The synthetic importer suite completes in well under one second on the current
Windows development machine after compilation, but this is not a manuscript
performance result. Small, image-heavy and near-limit corpus timings and peak
resident memory are pending.

The UI and headless code compile on Windows, and automated tests verify the
worker/cancellation/stale-generation and publication boundaries at the code
level. A manual Windows picker/report/import/reopen/offline smoke run has not
yet been recorded. macOS and Linux native menu, picker, cancellation, save and
reopen smoke checks are also pending. No untested platform is reported as
verified.

The DOCX implementation files pass `rustfmt`. During the workspace gate, an
unrelated pre-existing visual-edit change in `src/app/root_view.rs` had moved
the GPUI context into a closure and then reused it, while its new tests compared
`SharedString` values as `Arc`s and read an entity outside an app update. The
gate fix captures the entity handle before the closure, compares the cached
string pointers directly, moves reads into app updates, and applies rustfmt;
it does not change the visual-edit behavior.

## Release gate

The feature must not be described as fully producer-qualified until the real
20–30-file corpus, output reopen checks in Source/Split/Read/Visual Edit,
performance measurements, Windows smoke run, and macOS/Linux smoke runs are
recorded here. Automated checks and OpenSpec validation may pass while those
release-evidence tasks remain deliberately incomplete.
