# Design: improve-html-table-column-sizing

## Context

`html_table_grid_view` (src/app/preview.rs) renders the resolved `HtmlTableGrid` as one GPUI CSS grid: every non-spacer cell gets explicit `col_start`/`row_start`/`col_span`/`row_span`, and the container gets `.grid_cols(n)` — n equal `minmax(0, 1fr)` tracks. Spans are handled natively and correctly; column *widths* are not.

Reported case (cover sheet exported to HTML):

```html
<table>
<tr><th rowspan="5"></th><th colspan="3"></th><th></th></tr>
<tr><td colspan="3">瀚博载天VA16 AIGC大模型训推一体加速卡</td><td rowspan="4"></td></tr>
<tr><td colspan="3">测试报告</td></tr>
<tr><td>文档版本</td><td colspan="2">01</td></tr>
<tr><td>发布日期</td><td colspan="2">2026-08-10</td></tr>
…
</table>
```

Columns 0 and 4 hold only empty cells. Equal tracks give each 20% of the width; the browser's auto layout collapses them to padding width. Additionally the all-empty `<th>` row 1 and the empty spanning `<th>`s take pipe-table header shading (gray band + frame) that browsers never draw for this markup.

## Goals / Non-Goals

**Goals**

1. Column widths track content, so empty/near-empty columns shrink and content columns expand.
2. All-empty header rows stop rendering with header emphasis.
3. No new derived-state surface; no regression for data tables whose columns are all populated (they stay ~equal, which matches their content distribution).

**Non-Goals:** full `table-layout: auto` fidelity, `<colgroup>`/`width` attributes, `align`/`valign`, borderless HTML tables.

## Decision 1 — weighted `fr` tracks via a vendored-gpui patch

gpui 0.2.2 exposes only `grid_cols(count)` (`Style.grid_cols: Option<u16>` → taffy `repeat(count, minmax(0, 1fr))`). Rejected alternatives:

- *Nested flex rows + spacer struts* (the pre-CSS-grid design): breaks `rowspan` cell heights/borders across rows — the reason the code moved to CSS grid.
- *Micro-track emulation* (N equal tracks, cells spanning proportional ranges): N×rows grid areas per table in taffy's grid solver; layout cost scales badly and col/row coordinates are `i16`-bounded.
- *Percentage widths on cells*: in CSS grid the track, not the item, wins; percentages resolve against the spanned tracks, not vice versa.

Patch (mirrors the existing Markion f32-suffix patch in `to_grid_repeat`):

- `Style.grid_col_weights: Option<Vec<f32>>` (after `grid_cols`), `None` in `Default`.
- `to_taffy`: `grid_template_columns` = per-weight `minmax(length(0.), fr(w))` when set, else the existing `to_grid_repeat(&self.grid_cols)`.
- `Styled::grid_col_weights(self, weights: Vec<f32>) -> Self`.

Additive only; `grid_cols` callers are untouched.

## Decision 2 — weight function approximating auto layout

`html_table_column_weights(grid: &HtmlTableGrid) -> Vec<f32>` (pure, GPUI-free, in `src/parse.rs` next to the grid model):
- Per column `c`: `weight[c] = max over cells covering c of (min(score, CAP) / colspan)`. Max, not sum: a column needs to fit its widest single cell; stacked narrow cells must not inflate it (mirrors how browsers distribute max-content widths).
- `score(text)` = Σ glyph widths: CJK/full-width glyphs count 2.0, others 1.0 (dependency-free range check), plus `IMAGE_SCORE = 12.0` per cell image (intrinsic size is resolved later at paint).
- Two guards keep banners from skewing the layout: `CAP = 24` (longer cells wrap inside their span instead of demanding width) and cells whose `colspan` covers **all** columns contribute nothing (a full-width banner — the Word cover-sheet copyright row — can always wrap within the whole table and constrains no individual column).
- Empty-column floor: after the max pass, floor every weight at `(max_weight * 0.1).clamp(0.75, 3.0)`; empty columns become narrow but visible slivers so drawn borders do not overlap. If `max_weight == 0` (no scorable content anywhere) return equal weights — identical to today's layout.
- The function is total: returns exactly `columns.max(1)` finite positive weights.

Complexity: O(total cells × colspan) — trivial next to grid parsing.

## Decision 3 — header emphasis needs visible header content

`html_table_row_has_visible_header(row: &[HtmlTableCell]) -> bool`: true iff some non-spacer `is_header` cell has non-whitespace text or an image. `html_table_grid_view` applies semibold + header background per cell only when `cell.is_header && row_has_visible_header`; otherwise the cell renders as body. Effect: the classic matrix corner `<th></th><th>A</th>` keeps its shaded row, while an all-empty cover-sheet `<th>` frame renders as plain body cells. The grid model (`is_header`) and all export paths are unchanged — this is presentational policy in the renderer, fed by the pure predicate.

## Data flow / caching

`compute_preview_blocks` (per version, cached, `Arc`) → `PreviewBlock::Html` → `HtmlPreviewPart::Table { grid }` → at render time `html_table_column_weights(grid)` + `html_table_row_has_visible_header(row)`. Both are pure functions of the cached grid; computing them per frame is O(cells) arithmetic (no text shaping, no allocation beyond one `Vec<f32>`). If profiling ever warrants, the weights can be stored in `HtmlTableGrid` when built — deliberately not done now to keep the model minimal.

## Risks

- **taffy `fr` distribution with `row_span`**: taffy 0.9's grid distributes spanning items' intrinsic sizes across tracks; weights only affect column sizing, orthogonal to row spans. Worst case is a slightly different column ratio, never misplacement (explicit `col_start`/`col_span` pin every cell).
- **Vendor patch drift**: one additive field; conflicts on upstream gpui merges are mechanical.
- **Verification**: no Rust toolchain is available in the authoring environment; compile/test must run on the user's machine or CI (`cargo test --workspace`).
