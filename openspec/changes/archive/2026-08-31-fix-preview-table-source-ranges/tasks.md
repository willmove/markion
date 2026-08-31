# Tasks — fix-preview-table-source-ranges

Implements `specs/markdown-editing/spec.md` and `specs/tables-outline/spec.md` per `design.md`.
Each group is sized to a single testable commit. Preview table ranges stay inside the existing per-document-version `Arc<Vec<PreviewBlock>>` derivation — no new cache, no keystroke reparse.

## 1. Failing regression (incident shape)

- [x] 1.1 Add a compact fixture in `src/lib.rs` tests: H2 + blank + H3, two ordinary multi-column tables, two one-column `| command |\n| --- |` tables, then two later `| Dies | Throughput |` tables. Do not check in the 634 KB source document.
- [x] 1.2 Assert on `derive_preview_and_outline` / `preview_blocks()`: every `PreviewBlock::Table` has a non-empty `source_range`; `source[range]` contains that table’s header line; tables appear in authored order; no table sits between the leading H2 and H3; Dies headers are not at offset `0`.
- [x] 1.3 Confirm the new test fails on current `main` (one-column tables steal ranges; Dies tables land at `0..0`).

## 2. Parser assigns event source ranges

- [x] 2.1 In `MarkdownDocument::derive_preview_and_outline`, assign `PreviewBlock::Table.source_range` from the pulldown-cmark `End(Table)` event range (`body_offset + range`). Stop zipping `table_ranges_fn(text)` in this function.
- [x] 2.2 Use that same event-range start for `item_nested_block_start` when a table is nested in a list item.
- [x] 2.3 Delete the `unwrap_or(0..0)` table fallback. A table with rows always has an event range; do not emit an empty placeholder that sorts to the document head.
- [x] 2.4 Keep the existing `sort_by_key(|block| block.source_range().start)` as a nested-block order restore only. Re-run the tests from 1.x — they MUST pass. Per-version `Arc` preview caching is unchanged.

## 3. Editing lookup stays on `table_ranges`

- [x] 3.1 Update `table_ranges_track_multiple_source_tables` so preview table ranges are no longer required to equal `table_ranges()` byte-for-byte. Preview ranges cover parser-emitted tables (including one-column); `table_ranges()` remains the two-or-more-column editing index.
- [x] 3.2 Add or extend a test that `edit_table_at` / Visual Edit toolbar targeting still mutates the caret’s two-column table after the range-source change (not a neighbor table).
- [x] 3.3 Re-run existing nested-list-with-table tests (document-ordered block stream). Adjust only if event-range vs old `table_ranges` newline ownership differs, keeping list-item truncation before the nested table.

## 4. Validation

- [x] 4.1 `cargo test --workspace` passes with the new and updated tests.
- [x] 4.2 `openspec validate fix-preview-table-source-ranges` passes.
