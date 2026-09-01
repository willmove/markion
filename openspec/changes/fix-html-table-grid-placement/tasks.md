## 1. Placement helper (`src/parse.rs`)

- [x] 1.1 Add GPUI-free `html_table_grid_line_end(start: i16, span: u16) -> i16` (`start.saturating_add(span as i16)`) next to the HTML table grid model; document that exclusive CSS/GPUI end lines MUST be used instead of `col_span`/`row_span` (those wipe `col_start`/`row_start`)
- [x] 1.2 Test the reported cover-sheet HTML: after `parse_html_table_grid`, `文档版本` and `01` occupy different columns of the same row with non-overlapping exclusive line ranges from `html_table_grid_line_end`; pin the `12 V` `rowspan="3"` datasheet so its column range is unchanged
- [x] 1.3 Re-export the helper through `src/lib.rs` and `src/app/mod.rs` if the renderer imports via `use super::*`

## 2. Renderer placement and empty cells (`src/app/preview.rs`)

- [x] 2.1 In `html_table_grid_view`, place each non-spacer cell with `col_start`/`col_end`/`row_start`/`row_end` from the existing walk + `html_table_grid_line_end`; do not call `col_span` or `row_span` on those items
- [x] 2.2 Add `.w_full()` on the grid container; add `.min_w_0()` on content cells so text wraps inside the track
- [x] 2.3 Empty paint-spacers (non-parser-spacer, no image, trimmed text empty): no padding, no fill, no internal right/bottom stroke; content cells keep current padding, fill, and strokes

## 3. Visual Edit chrome

- [x] 3.1 When `visual_html_editor` presents HTML whose `html_preview_parts` include a `Table`, use a bare collapsible wrapper (no `border_1`/`rounded_md`); keep hover `</>` and caret-in-block source payload
- [x] 3.2 HTML blocks without a table part keep the existing bordered collapsible chrome

## 4. Validation

- [x] 4.1 `cargo test --lib` covering the new `html_table_grid_line_end` / cover-sheet column tests and existing `html_table_tests` (`rowspan_three_places_cell_across_rows`, `cover_table_weights_content_over_empty_spacers`)
- [ ] 4.2 Manual check of the reported cover-sheet table in **Read** and **Visual Edit**: `文档版本` and `01` are two columns; empty struts are not a stack of blank cards; Visual Edit is one table grid plus optional source payload, not a card per row
- [x] 4.3 `openspec validate fix-html-table-grid-placement`
