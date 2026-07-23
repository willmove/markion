## 1. Data model: RichText table cells

- [x] 1.1 Add `From<String>` / `From<&str>` ergonomics to `RichText` (and a `RichText::plain` helper) so existing `"...".into()` call sites keep compiling when the target type becomes `RichText`.
- [x] 1.2 Change `PreviewBlock::Table.rows` from `Vec<Vec<String>>` to `Vec<Vec<RichText>>` in `src/model.rs`; update `source_range()` match (no change needed, already `..`).
- [x] 1.3 Change `VisualBlockKind::Table.rows` from `Vec<Vec<String>>` to `Vec<Vec<RichText>>` in `src/model.rs`.

## 2. Parsing: accumulate inline spans per cell

- [x] 2.1 Change `TableDraft.current_cell` (`src/table.rs`) from `String` to `Vec<InlineSpan>`; update `TableDraft::default`.
- [x] 2.2 Update `push_preview_rich` (`src/parse.rs`) table branch to call `append_span`/`append_extended_text` into `table.current_cell` instead of `push_str`, mirroring the heading/paragraph path.
- [x] 2.3 Update `push_preview_math` (`src/parse.rs`) table branch to push a math `InlineSpan` into `table.current_cell` instead of `push_str`.
- [x] 2.4 Update `Event::Start(Tag::TableCell)` / `End(TagEnd::TableCell)` handling (`src/lib.rs`) to clear/finalize the span vector: on end, call `finish_rich_text` (or a trim-only variant) and push the resulting `RichText` into the current row.
- [x] 2.5 Verify extended inline syntax (`==highlight==`, emoji, bare autolinks) renders inside cells since the table branch now reuses `append_extended_text`.

## 3. Preview rendering

- [x] 3.1 Update the `PreviewBlock::Table` branch in `preview_block_view` (`src/app/preview.rs`) to render each cell with `rich_text_element` instead of `selectable_plain_text`, passing the cell's `RichText`.
- [x] 3.2 Verify link cells are clickable (open in browser) in Preview/Read/Split modes via the `with_links` path already in `rich_text_element`.

## 4. Exporters: plain-text compatibility

- [x] 4.1 Update `render_docx_table` (`src/export.rs`) to read `cell.text` instead of the `String` directly.
- [x] 4.2 Update the LaTeX table renderer (`render_latex_table` in `src/render.rs`) to read `cell.text`.
- [x] 4.3 Confirm HTML export is unaffected (uses `crates/markdown` AST path, not `PreviewBlock`).

## 5. Visual Edit: per-cell inline runs and reveal

- [x] 5.1 Add fields to `VisualTableCell` (`src/model.rs`) for per-cell `editable_runs: Vec<VisualInlineRun>` and `reveal_groups: Vec<VisualRevealGroup>` (or a parallel lookup structure).
- [x] 5.2 In `visual_block_editor` (`src/visual.rs`), for each `TableCellSourceRange`, call a per-cell inline parse (extract `inline_runs` logic so it can run on a single cell's authored text slice) and store results on the corresponding `VisualTableCell`.
- [x] 5.3 Extend `visual_editor_field_projection` (`src/app/preview.rs`) to accept the cell's runs/reveal groups and the caret-active flag. When unfocused, build `VisualProjectionSpan`s with real `InlineStyle`/`link` from rendered runs (visible text = resolved text). When focused (caret owns the cell), treat the entire cell source range as a revealed `Source` piece (visible text = authored markup).
- [x] 5.4 In `visual_editor_field_element` (`src/app/preview.rs`), convert `projection.spans` into `HighlightStyle`s (reuse the conversion from `visual_text_element`) and construct `StyledText::new(text).with_highlights(highlights)` to pass into `VisualEditableText` for table cells (when `styled_text` is `None`).

## 6. Visual Edit: view wiring

- [x] 6.1 Update `visual_table_view` signature (`src/app/preview.rs`) from `rows: &[Vec<String>]` to `rows: &[Vec<RichText>]`; update the call site in `visual_block_view` (`preview.rs:2321`).
- [x] 6.2 Pass the per-cell runs/reveal groups from the `VisualBlockEditor::Table` cells into `visual_editor_field_element` for each cell.

## 7. Tests

- [x] 7.1 Update the existing table-preview test (`src/lib.rs:2958`) to assert `RichText`-based cells instead of `String`.
- [x] 7.2 Add a parsing test asserting `**bold**` and `[text](url)` inside a table cell produce `InlineSpan`s with `bold=true` / a link target.
- [x] 7.3 Update `src/app/tests.rs` table test (`tests.rs:540`) for the `RichText` model change.
- [x] 7.4 Add a Visual Edit test asserting an unfocused cell renders styled spans and a focused cell reveals source (mirror existing `visual_text_element` reveal tests).
- [x] 7.5 Run `cargo test --workspace` and fix any remaining compile/test breakage.

## 8. Validation and archive

- [x] 8.1 Run `openspec validate render-table-cell-inline-formatting`.
- [x] 8.2 Run `openspec doctor`.
- [x] 8.3 Archive the change via `openspec archive render-table-cell-inline-formatting` after all tasks pass.
