## Context

`MarkdownDocument::derive_preview_and_outline` walks pulldown-cmark offset events and emits `PreviewBlock`s. Most block kinds take `source_range` from that event. Tables do not: a parallel `table_ranges(text)` scan is zipped in document order, and `End(Table)` does `table_ranges.next().unwrap_or(0..0)`.

`table_ranges` / `is_markdown_table_candidate` require at least two cells per line. GFM (and pulldown-cmark with `ENABLE_TABLES`) also emit one-column tables such as:

```markdown
| vasmi setconfig dpm=enable -d all |
| --- |
```

Those events still consume the iterator. Later multi-column tables then receive the wrong range or `0..0`. A stable sort by `source_range().start` (needed so list-nested fences/tables appear in document order) then parks `0..0` tables at the start of the file — after the first heading, which also starts at offset 0.

That derived `Arc<Vec<PreviewBlock>>` is the per-document-version cache. Visual Edit, Split Preview, Read mode, export, and sync scroll all read it, so the hoist is not Visual-Edit-specific.

```
source
  │
  ├─ pulldown-cmark offset events
  │     End(Table) rows ──► PreviewBlock::Table.content
  │     table_ranges.next() ──► PreviewBlock::Table.source_range   ← broken zip
  │     sort_by start ──► hoisted 0..0 tables
  │
  └─ table_range_at(caret) ──► cell edit / toolbar (independent, keep)
```

## Goals / Non-Goals

**Goals:**

- Every `PreviewBlock::Table` carries a non-empty source range covering the same bytes as the pulldown-cmark table event that produced its rows.
- Mixed one-column and multi-column GFM tables appear in authored order in every consumer of the preview stream.
- Nested-in-list table ordering still relies on sorting real ranges, not on inventing `0..0`.
- Cell editing and toolbar targeting keep using `table_range_at` / `table_ranges` as a caret lookup, not as the preview-block range source.

**Non-Goals:**

- Making one-column tables cell- or toolbar-editable (`table_cell_source_ranges` already rejects `< 2` columns).
- Changing HTML `<table>` handling, incremental `split_regions`, or Visual Edit caret/viewport behavior.
- Teaching `table_ranges` to mimic pulldown-cmark’s full GFM table grammar.

## Decisions

### 1. Preview table ranges come from the parser event

On `Event::End(TagEnd::Table)`, assign `source_range` from the offset iterator (already computed as `body_offset + range`) — the same path headings, paragraphs, and fences use. Stop calling `table_ranges_fn` inside `derive_preview_and_outline`.

Use that same event start for `item_nested_block_start` so a list item still truncates before a nested table.

**Alternative considered:** widen `is_markdown_table_candidate` to one-column rows so the zip stays 1:1 on the incident document. Rejected: other accepted-by-pulldown / skipped-by-scan mismatches (pipe-containing prose glued to a table, parse_markdown_table rejects) would still exhaust the iterator and resurrect `0..0`.

**Alternative considered:** zip when the next `table_ranges` entry overlaps the event range, otherwise use the event range. Rejected: two grammars remain coupled; the overlap heuristic is another silent footgun.

### 2. Ban empty placeholder ranges

Remove `unwrap_or(0..0)` for table blocks. A table with rows always has an event range. If a future path lacks an event range, skip emitting the block rather than emitting `0..0` (which sorts to the document head).

The existing `sort_by_key(|block| block.source_range().start)` stays. It is only a nested-block order restore; it must not be the mechanism that “fixes” invented ranges.

### 3. Keep `table_ranges` for editing lookup only

`MarkdownDocument::table_ranges`, `table_range_at`, `edit_table_at`, and the Visual Edit toolbar continue to resolve “which source table contains this caret” through the dedicated scan. That scan may still ignore one-column tables; those tables render in the right place and, if cell bounds cannot be proven, stay under the existing conservative table-edit fallback.

Existing test `table_ranges_track_multiple_source_tables` currently asserts preview `source_range` equals `table_ranges()`. After this change they may differ by a trailing newline or by including one-column tables only on the preview side. Split the assertion: preview ranges match event-covered table bytes; `table_ranges()` remains a 2+-column editing index.

### 4. Regression fixture, not the 634 KB file

Tests use a small synthetic document with the incident shape: two ordinary multi-column tables, then one-column `| command |\n| --- |` tables, then later `Dies | Throughput` tables, with a heading pair and a blank line at the top. Assert:

- no `PreviewBlock::Table` between the first H2 and the following H3;
- four Dies tables sit at source offsets matching their authored rows;
- one-column command tables sit in §2, not in §3;
- every table `source_range` is non-empty and `source[range]` contains that table’s header line.

Derivation remains once per document version, stored on the existing `Arc<Vec<PreviewBlock>>`. No new cache, no keystroke reparse.

## Risks / Trade-offs

- **[Risk] pulldown table ranges include a trailing newline that `table_ranges()` omitted** → `table_preview_source_range` keeps the event *start* (so `0..0` hoisting cannot return) and trims trailing CR/LF so Visual Edit cell mapping and whole-table replacement still see table lines only. The newline after the last row stays a gap/whitespace row, matching the previous `table_ranges()` end.
- **[Risk] `edit_table_at(table.source_range.start)` fails if the event start is not a table-candidate line** → Mitigation: pulldown’s table start is the first header `|`; `table_range_at` already accepts that. Add a test that Visual Edit toolbar targeting still finds a 2-column table after the range source change.
- **[Risk] one-column tables become visible grids where authors meant fenced commands** → That is GFM. They already rendered (just in the wrong place). Not a regression of this fix.

## Migration Plan

In-memory derived state only. No document format, undo snapshot, or preferences change. Rollback is a revert of the parse-arm change.

## Open Questions

None. Cell-editability of one-column tables stays a future tables-outline change if anyone wants it.
