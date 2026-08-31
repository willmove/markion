## 1. Column-weight helper

- [x] 1.1 Add a GPUI-free `table_column_flex_weights(rows, table_font_size)` helper in `src/table.rs` that estimates each cell from rendered `RichText.text` (ASCII `0.55 * font_size`, other scalars `1.0 * font_size`, plus 16px padding), takes the per-column max, and floors empty columns at `padding + 2 * table_font_size`
- [x] 1.2 Cover the helper with unit tests: a 名称/说明-style table gives the short column a strictly smaller weight; one-column tables return a single positive weight; empty/ragged columns stay at the floor; CJK cells count wider than the same number of ASCII letters; weights do not depend on Markdown source markup

## 2. Shared table cell flex

- [x] 2.1 Extract a small cell-style helper in `src/app/preview.rs` that applies `flex_grow = weight`, `flex_basis = 0`, `flex_shrink = 1`, and `min_w_0` in place of `.flex_1()`, so Visual Edit and Read/Split cannot drift
- [x] 2.2 Compute weights once per table from the cached rows plus `typography.table_font_size` at paint time; do not store weights on `PreviewBlock`/`VisualBlock` or bump document version

## 3. Wire both surfaces

- [x] 3.1 Apply the weighted cell style in the Read/Split `PreviewBlock::Table` branch
- [x] 3.2 Apply the same weights in `visual_table_view` so Visual Edit matches Read mode; keep sizing on rendered cell text when a focused cell reveals source markup
- [x] 3.3 Add tests that Visual Edit and preview request identical weights for the same rows, and that existing table toolbar/cell-edit tests still pass
- [x] 3.4 Run `cargo test` for the affected table and preview tests

## 4. Column-share cap and header wrap budget

- [x] 4.1 Raise header-column minima in `table_column_flex_weights` for short 1–3 character parenthesis units (`()` / `（）`) and for a three-line header wrap budget; compress body extras with `sqrt(extra × floor)` so long paragraphs cannot linearly starve those mins
- [x] 4.2 Clamp each column to `3 / n` of the weight sum (never below its header min; skip for one-column tables)
- [x] 4.3 Tests: a six-column 实际功率（W） table keeps every share ≤ 1/2 and gives the unit header a larger share than an uncapped linear split; short paren units set a min at least as wide as `（W）`; a long header min is at least one third of its unwrapped width; 名称/说明 stays unequal
- [x] 4.4 Run `cargo test --lib table::tests` and `cargo test visual_and_preview_tables_share`
