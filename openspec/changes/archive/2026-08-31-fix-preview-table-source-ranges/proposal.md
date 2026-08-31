## Why

A real VA16 test-guide document renders GFM tables in the wrong place: four later `Dies | Throughput` result tables jump into the blank line between `## 1. 前言` and `### 1.1 目的`, while §2.3 one-column command tables vanish and §3.1–3.3 “输入长度” tables occupy the Dies slots. The source file is correct. The derived preview stream assigns table *content* from pulldown-cmark events but *source ranges* from a parallel `table_ranges()` scan that silently skips one-column tables; leftover tables then get `0..0`, and a document-order sort pins them at the start of the file. Visual Edit, Split Preview, Read mode, export, and sync scroll all consume that stream.

## What Changes

- Give every `PreviewBlock::Table` the source range of the pulldown-cmark table event that produced it (the same contract headings, paragraphs, and fences already follow). Stop zip-assigning ranges from `table_ranges()`.
- Never fall back to an empty `0..0` range for a table that has rows. An exhausted or mismatched range iterator MUST NOT invent a range that sorts before real document content.
- Keep `table_ranges()` / `table_range_at()` as the lookup used by cell editing and toolbar targeting; do not require that scan to be 1:1 with parser table events.
- Add regression coverage for mixed one-column and multi-column GFM tables: block content, ordering, and source offsets must match the authored source; the incident fixture shape (normal tables, then `| command |\n| --- |` tables, then later result tables) must not hoist or shuffle.

Non-goals: changing GFM table cell-editing semantics, making one-column tables toolbar-editable, altering HTML `<table>` rendering, Visual Edit caret/viewport work, or incremental region splitting.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: derived preview blocks for GFM tables SHALL carry the parser event’s source range (or an equivalent non-empty range over the same bytes) and SHALL appear in document order. Empty placeholder ranges are forbidden.
- `tables-outline`: GFM tables of any column count that the CommonMark+GFM parser emits SHALL render at their authored source position in Split Preview, Read mode, and Visual Edit — including one-column header-and-separator tables mixed with later multi-column tables.

## Impact

- `src/lib.rs` — `MarkdownDocument::derive_preview_and_outline` table `End` arm; the post-parse `sort_by_key` over `source_range().start` stays, but only as a nested-block order restore, not as a repair for invented ranges.
- `src/table.rs` — no required change to cell-edit helpers; they remain the targeting path. Optional comment/test clarifying they are not the preview-block range source.
- Tests in `src/lib.rs` (and table unit tests if the 1:1 zip assertion in `table_ranges_track_multiple_source_tables` needs a sibling for one-column tables).
- Architecture invariants: table ranges are still computed once per document version inside the existing `Arc`-shared preview derivation. No per-keystroke reparse, no new cache surface, no `gpui` in `crates/*`.
- Localization: none (no new user-facing strings).
