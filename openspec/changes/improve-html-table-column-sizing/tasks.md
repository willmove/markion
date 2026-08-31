# Tasks — improve-html-table-column-sizing

## 1. Vendored gpui: weighted grid column tracks

- [x] 1.1 Add `Style.grid_col_weights: Option<Vec<f32>>` (field + `Default::None`) in `vendor/zed/crates/gpui/src/style.rs`
- [x] 1.2 Map it in `to_taffy` (`vendor/zed/crates/gpui/src/taffy.rs`): when set, `grid_template_columns` = one `minmax(length(0.), fr(w))` per weight via a `to_weighted_grid_columns` helper; else keep `to_grid_repeat(&self.grid_cols)`
- [x] 1.3 Add `Styled::grid_col_weights(weights: Vec<f32>)` in `vendor/zed/crates/gpui/src/styled.rs` mirroring `grid_cols`

## 2. Grid model helpers (`src/parse.rs`)

- [x] 2.1 Add pure `html_table_column_weights(&HtmlTableGrid) -> Vec<f32>`: max-per-column of `min(score, 24) ÷ colspan`, CJK glyphs ×2, image score 12, full-width-banner cells (`colspan == columns`) skipped, floor `(max*0.1).clamp(0.75, 3.0)`, equal fallback when nothing is scorable — GPUI-free, total
- [x] 2.2 Add pure `html_table_row_has_visible_header(&[HtmlTableCell]) -> bool`
- [x] 2.3 Fix the stale `has_rowspan` doc comment (no renderer uses a fixed row height: GPUI uses native `row_span`, DOCX `vMerge`, PDF/LaTeX drop spans)
- [x] 2.4 Tests: cover-table weights (empty spacer columns ≪ content columns, full-width banner constrains nothing); all-empty table → equal weights; banner-free control table equality; matrix corner vs cover frame for the header predicate; image-bearing column outranks empty

## 3. Renderer (`src/app/preview.rs`)

- [x] 3.1 `html_table_grid_view`: replace `.grid_cols(columns)` with `html_table_column_weights` tracks (function's equal fallback covers the degenerate case)
- [x] 3.2 Header emphasis (semibold + `header_bg`) applies only when `cell.is_header && html_table_row_has_visible_header(row)`; otherwise body styling
- [x] 3.3 Re-export the helpers through `src/lib.rs` and `src/app/mod.rs` so `preview.rs` sees them via `use super::*`

## 4. Validation

- [ ] 4.1 `cargo build` clean (vendored gpui patch compiles)
- [ ] 4.2 `cargo test --workspace` green, including the new `html_table_tests`
- [ ] 4.3 Manual visual check in Visual Edit with the reported 瀚博 cover-sheet document: empty side columns collapse to slivers, no gray header frame, content columns dominant
- [ ] 4.4 `openspec validate improve-html-table-column-sizing`
