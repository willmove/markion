## 1. Column-weight helper

- [ ] 1.1 Add a GPUI-free `table_column_flex_weights(rows, table_font_size)` helper in `src/table.rs` that estimates each cell from rendered `RichText.text` (ASCII `0.55 * font_size`, other scalars `1.0 * font_size`, plus 16px padding), takes the per-column max, and floors empty columns at `padding + 2 * table_font_size`
- [ ] 1.2 Cover the helper with unit tests: a 名称/说明-style table gives the short column a strictly smaller weight; one-column tables return a single positive weight; empty/ragged columns stay at the floor; CJK cells count wider than the same number of ASCII letters; weights do not depend on Markdown source markup

## 2. Shared table cell flex

- [ ] 2.1 Extract a small cell-style helper in `src/app/preview.rs` that applies `flex_grow = weight`, `flex_basis = 0`, `flex_shrink = 1`, and `min_w_0` in place of `.flex_1()`, so Visual Edit and Read/Split cannot drift
- [ ] 2.2 Compute weights once per table from the cached rows plus `typography.table_font_size` at paint time; do not store weights on `PreviewBlock`/`VisualBlock` or bump document version

## 3. Wire both surfaces

- [ ] 3.1 Apply the weighted cell style in the Read/Split `PreviewBlock::Table` branch
- [ ] 3.2 Apply the same weights in `visual_table_view` so Visual Edit matches Read mode; keep sizing on rendered cell text when a focused cell reveals source markup
- [ ] 3.3 Add tests that Visual Edit and preview request identical weights for the same rows, and that existing table toolbar/cell-edit tests still pass
- [ ] 3.4 Run `cargo test` for the affected table and preview tests
