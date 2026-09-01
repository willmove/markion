## Context

`parse_html_table_grid` already resolves the reported cover sheet into a 5-column `HtmlTableGrid` with spacers for `rowspan`. `html_table_grid_view` (`src/app/preview.rs`) is supposed to pin every non-spacer cell with GPUI CSS grid. It chains:

```
.col_start(c).row_start(r).col_span(colspan).row_span(rowspan)
```

GPUI `Styled::col_span` replaces the **entire** column placement with `Span(n)..Span(n)` (same for `row_span`). Taffy then treats both ends as spans, drops the end span, and **auto-places** the item. Side-column `rowspan` occupancy is no longer reserved, so later cells (including `文档版本` and `01`) stack as full-width bands. Confirmed in Read mode, not only Visual Edit.

Empty cells still get 8px padding and right/bottom strokes, so Word spacer `<th>`/`<td>` paint as blank cards. Visual Edit additionally wraps the presentation in `visual_collapsible_source_block` (`border_1` + `rounded_md` + `overflow_hidden`), a second table-like chrome around the same grid.

`improve-html-table-column-sizing` already feeds `grid_col_weights` into this view. Those weights only matter once items sit on the intended tracks.

```
MarkdownDocument.text
        │  per version, Arc-cached
        ▼
PreviewBlock::Html { html }
        │  html_preview_parts at paint (pure parse of cached html)
        ▼
HtmlPreviewPart::Table { grid }     ← unchanged
        │
        ▼
html_table_grid_view
  place cells on exclusive grid lines   ← this change
  empty cells: occupy tracks, no card
  .w_full() + existing column weights
        │
        ├─ Read / Split Preview: table chrome only
        └─ Visual Edit: collapsible source without a second table border
```

No new derived-state surface. Grid parse stays inside the per-version HTML block.

## Goals / Non-Goals

**Goals:**

1. Every rendered HTML table cell occupies the parser's column/row span, in Read, Split Preview, and Visual Edit.
2. Two content cells in one row (e.g. `文档版本` | `01`) paint as separate columns, not one concatenated run.
3. Empty spacer cells do not paint as padded cards; `rowspan`/`colspan` geometry still holds.
4. Visual Edit does not wrap a table grid in a second rounded table border.

**Non-Goals:**

- Parser/`HtmlTableGrid`/export/GFM tables.
- Visual Edit cell editing, `colgroup`/`width`/`align`.
- Borderless HTML tables in general.
- Vendor patch to GPUI `col_span` (app can avoid the helper).
- Merging with `improve-html-table-column-sizing`.

## Decisions

### D1: Exclusive grid lines, never `col_span`/`row_span` after start

CSS / Taffy want `grid-column: start-line / end-line` with **exclusive** end lines (1-based, matching GPUI `col_start`).

```
col_end = col_start + colspan
row_end = row_start + rowspan
```

A small GPUI-free helper (next to the grid model in `src/parse.rs`) maps `(start, span) -> end` so tests can pin the cover-sheet cells without standing up GPUI:

`html_table_grid_line_end(start: i16, span: u16) -> i16`

The renderer MUST set `col_start`/`col_end`/`row_start`/`row_end` from the same walk it already uses (skip spacers, 1-based `col`/`row_index+1`). It MUST NOT call `col_span` or `row_span` on those items.

**Alternatives considered:** (a) patch GPUI so `col_span` sets only `column.end = Span(n)` — rejected, vendor drift for one call site; (b) go back to flex rows + spacer struts — rejected, that is why the code moved to CSS grid; (c) rely on auto-placement + occupancy — rejected, Taffy auto-placement is what produced the bug.

### D2: Definite width on the grid container

`.w_full()` on the grid root so `minmax(0, fr(weight))` tracks resolve against the preview/visual content column. Without a definite width, `fr` tracks can collapse under `overflow_hidden` parents (Visual Edit collapsible; also a Read-mode shrink-wrap risk).

Cells with content get `min_w_0` so long CJK strings wrap inside the track instead of overflowing into the next cell (which also looks like concatenation).

### D3: Empty content cells occupy tracks but do not paint as cards

Skip drawing remains limited to **parser spacers** (`is_spacer`, covered by an earlier `rowspan`). Real empty `<td>`/`<th>` still become grid items so the span footprint is explicit.

A cell is an empty spacer for **paint** when it is not a parser spacer, has no image, and `content.text` is empty after trim. Those items:

- keep `col_start`/`col_end`/`row_start`/`row_end`
- take **no** content padding
- take **no** right/bottom internal stroke
- take **no** header/body fill (transparent)

Content cells keep today's padding, fills, and internal strokes. Outer table `border_1` + `rounded_md` stays for the grid as a whole.

**Alternatives considered:** (a) omit empty cells from the view tree — rejected, then `rowspan` holes depend on auto-placement again; (b) hide entire empty rows — rejected, a leading empty header row plus a left `rowspan=5` strut would fight; (c) borderless tables globally — rejected, datasheets still want pipe-table chrome on content cells.

### D4: Visual Edit — one table chrome

`visual_html_editor` keeps the collapsible `</>` payload. When `html_preview_parts(html)` contains a `Table` part, the collapsible wrapper SHALL NOT add `border_1` / `rounded_md` (bare: `relative` + hover toggle + optional payload). The grid's own border is the surface, matching Read.

HTML blocks that are only text/images keep the current bordered collapsible.

Parse of parts already happens inside `html_preview_block_view`; implementation may parse once and reuse, or call `html_preview_parts` twice (cheap, cached html string).

## Risks / Trade-offs

- **[Risk] Exclusive end lines off-by-one** → Mitigation: helper + unit test on the cover sheet: `文档版本` at column index 1 (0-based) `colspan=1`, `01` at column 2 `colspan=2`; 1-based lines `2..3` and `3..5` must not overlap.
- **[Risk] Zero-padding empty cells make a `rowspan` strut invisible** → Mitigation: desired for cover sheets; datasheet empty body cells also lose the inner stroke — acceptable; content cells still draw the shared edge when they sit next to an empty cell (`border_r` on the content cell). If both neighbors are empty, that internal line disappears (cover-sheet pattern).
- **[Risk] Bare collapsible makes non-table HTML in the same block lose a box around leading text** → Mitigation: only drop wrapper chrome when a Table part is present; surrounding Text/Image parts sit above/below the table without an extra card, which matches Read.
- **[Trade-off]** Empty-column **weights** still floor to a sliver (`improve-html-table-column-sizing`). Placement fix does not change that; slivers without padding/stroke should be visually quiet.

## Migration Plan

Presentation-only. No document, settings, or API migration. Rollback = revert the preview/visual renderer commits.

## Open Questions

None — Read mode concatenation confirmed the bug is grid placement, not Visual-only chrome.
