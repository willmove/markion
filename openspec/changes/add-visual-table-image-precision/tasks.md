## 1. Column-width comment helpers

- [x] 1.1 Add GPUI-free parse/format/redistribute helpers in `src/table.rs` for `<!-- markion-cols:… -->` and verify unit tests cover valid percents, floor clamping, adjacent-pair drag, insert/delete column adjustment, and prefix skip
- [x] 1.2 Add `table_column_flex_weights_with_authored` so matching percents win over the content heuristic, and verify a 30/70 comment yields 30:70 weights while a mismatched count falls back

## 2. Absorb comments into table preview blocks

- [x] 2.1 After preview derivation sort, absorb a preceding column-width Html comment into the following Table (including quote children) and verify no Html island remains and `source_range` covers the comment
- [x] 2.2 Skip the comment prefix in `visual_block_editor` cell mapping and verify table cells still map to pipe-table bytes

## 3. Persist widths on drag commit and structural edits

- [x] 3.1 Add `MarkdownDocument::set_table_column_widths_at` and verify it inserts or replaces the comment in one mutation
- [x] 3.2 Keep `edit_table_at` AddColumn/DeleteColumn in sync with an existing matching comment in that same mutation, verified by unit tests
- [x] 3.3 Wire Visual Edit column handles plus tab-local drag overlay and workspace-row drop; verify overlay does not bump version and mouse-up is one undoable comment write; Read/Split use the same percents

## 4. In-cell inline format controls

- [x] 4.1 Extend `visual_selection_format_target_for_block` to accept a selection fully inside one `TableCell` field and verify the block context menu shows Bold for that selection
- [x] 4.2 Gate `apply_markdown_format` so cross-cell / `|`-crossing selections and non-inline formats inside a table are no-ops, verified by unit tests and a GPUI Bold wrap of one cell

## 5. Image drag-resize and 10–100% width

- [x] 5.1 Accept integer `width=` 10–100 in `split_presentation_title` / `compose_title` and verify 40 round-trips while 9 is rejected
- [x] 5.2 Add a Visual Edit drag handle on exactly mapped inline images with overlay-then-commit, and verify reference-style images still have no handle; mouse-up is one undoable presentation mutation

## 6. Docs and verification

- [x] 6.1 Update `docs/visual-editing-quality.md` and the README limitation bullets for cell format controls, column-width comments, and image drag-resize
- [ ] 6.2 Run `cargo fmt --all -- --check`, `openspec validate add-visual-table-image-precision`, and `cargo test --workspace` (fix regressions in table/image/format tests only)
