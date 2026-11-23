## 1. Render requests cover table cells

- [x] 1.1 In `src/app/math_render.rs`, make `ensure_math_renders` request every math span of `PreviewBlock::Table` rows (header and body) and of block-quote children with the same key prose uses; verify with a new GPUI test in `src/app/tests.rs` (`cargo test --bin markion table_cell_math`) asserting that a formula present only in a table cell reaches a non-Pending cache entry after `run_until_parked`, and that the same formula in a paragraph and in a cell leaves exactly one cache entry.

## 2. Read and Split Preview cells render formulas

- [x] 2.1 In `src/app/preview.rs` (`PreviewBlock::Table` branch of `preview_block_view`), route a cell that has a math span through `rich_text_with_math_element`, and apply the same image-or-math condition to the read-only Visual Edit cell branch that has no proven field; verify the new test finds the Read-mode atom by its `preview-math-…` debug bounds and `issue_36_table_cell_image_renders_in_visual_edit_and_read` still passes.
- [x] 2.2 Add a regression test that a Read-mode preview selection spanning a cell formula copies its complete authored `$...$` syntax; verify it passes alongside `math_atom_hit_testing_and_copy_preserve_complete_authored_source`.

## 3. Visual Edit cells render formulas

- [x] 3.1 Thread `display_scale` from `visual_block_view` through `visual_table_view` into a new `visual_table_cell_element` that owns the moved mixed-prose layout and otherwise delegates to `visual_editor_field_element`; verify `cargo check` is clean and the existing table tests (`cargo test --bin markion visual_table`, `issue_36_table_cell_image_renders_in_visual_edit_and_read`) still pass with no behavior change.
- [x] 3.2 Trigger the mixed layout for math and render each Ready math span as a baseline-aligned `visual_math_atom` (Pending/Error fall through to authored-source fragments), and give math atoms their debug selectors; verify a new test (`table_cell_math_renders_in_visual_edit`) finds the cell atom while unfocused, then finds source text instead once the cell owns the caret, with Keep inline syntax hidden off and on, and that the document version never changes.
- [x] 3.3 Add `visual_table_math_hit_target` under each cell formula atom; verify a new test clicks the trailing and leading halves and lands the caret at the formula's exact authored end and start inside the cell, a Shift-click from another cell extends the rectangular cell selection, and text, version, dirty state, and undo history stay unchanged.
- [x] 3.4 Cover a mixed cell (text, formula, and an inline image) and a formula-only cell; verify they lay out without overlap and that Tab/Up/Down still traverse the column using the existing navigation tests plus the new mixed-cell case.
- [x] 3.5 Let a cell formula atom narrow with its cell (`max-width: 100%` plus the raster's aspect ratio) in both surfaces, found during the real-window smoke when a seven-column table overflowed in Split Preview; verify `table_cell_math_narrows_with_its_cell_instead_of_overflowing` passes for Read mode and Visual Edit.

## 4. Documentation, validation, and smoke

- [x] 4.1 Update the coverage note in `docs/visual-editing-quality.md` so the matrix records that GFM cells render math while unfocused and keep whole-cell source reveal on focus; verify the note is present and agrees with the delta spec.
- [x] 4.2 Run a plain `cargo check` (no `--tests`) and `cargo test --bin markion`; verify both are green or any failure is reported with its output.
- [x] 4.3 GUI smoke on the reported six-column NoC table in Visual Edit (unfocused and focused cells), Read, and Split Preview; verify formulas render as typeset atoms, pending/failed formulas fall back to authored source, and no crash or layout jump occurs when a render completes.
- [x] 4.4 Run `openspec validate fix-table-cell-math-rendering` and `openspec doctor`; verify both report no errors.
