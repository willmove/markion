## Why

Visual Edit currently projects a blockquote that contains both prose and a list as one parent row covering the complete quote plus separate child list rows whose source ranges sit inside that parent. The overlap forces the child rows into raw-source islands, while smart-punctuation substitutions can independently force the parent into a source island, so list source appears twice instead of as one editable list inside the quote.

## What Changes

- Preserve blockquote paragraphs and nested list items as one ordered quoted flow instead of splitting prose from an appended child-list collection.
- Derive Visual Edit rows for quoted paragraphs and list items with disjoint, complete source ownership while carrying their blockquote nesting context.
- Render ordered, unordered, nested, and task-list rows inside the blockquote presentation without duplicating source or degrading supported rows to `Unsupported` source islands.
- Recognize composite quote/list prefixes such as `> 1. ` and `> - [x] ` so marker reveal, caret mapping, and structural editing remain source-exact.
- Keep authored straight quotes and other smart-punctuation candidates editable as rendered Visual Edit text rather than allowing punctuation substitution to turn an otherwise supported quoted flow into a whole-block source island.
- Add model, projection, incremental-equivalence, interaction, and rendered-window regressions for mixed blockquote/list documents, including the reported Chinese fixture.

Non-goals: general recursive rendering for every Markdown container combination, arbitrary non-identity entity decoding, or replacing `MarkdownDocument.text` as the canonical editable representation.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: require Visual Edit to preserve ordered blockquote child flow, render nested list items exactly once inside the quote, maintain disjoint source mappings and composite-prefix editing, and avoid whole-block source fallback for authored smart-punctuation candidates in supported prose.

## Impact

- `src/lib.rs`, `src/model.rs`, and `src/parse.rs`: blockquote preview derivation and ordered child representation.
- `src/visual.rs`: non-overlapping quoted leaf projection, quote-context metadata, composite prefixes, and source-faithful Visual Edit punctuation parsing.
- `src/app/preview.rs` and related interaction helpers: quoted-row grouping/styling and source-backed caret/selection behavior.
- `src/source_mapped.rs`, `src/document_memory.rs`, export/text consumers, and stable-ID reconciliation: recursive/ordered child shifting, accounting, and compatibility audits.
- Tests in `src/lib.rs`, `src/visual.rs`, `src/source_mapped.rs`, and `src/app/tests.rs`.
- `docs/visual-editing-quality.md`: support classification and verification evidence for mixed quoted flows and source-faithful Visual Edit punctuation.
- The existing per-document-version `Arc` caches and incremental/full-parse equivalence remain mandatory; the change must not introduce render-frame reparsing or GPUI dependencies into workspace member crates.
