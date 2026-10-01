## 1. Index-addressed table structure edits (model)

- [x] 1.1 Add `TableStructureEdit` to `src/model.rs` and `permute_column_percents` to `src/table.rs`; extract the shared normalize/format/column-comment/apply tail of `edit_table_at` into a private helper; verify existing table tests in `src/lib.rs` still pass with `cargo test --bin markion table` (lib tests) and `cargo test --lib table`
- [x] 1.2 Implement `MarkdownDocument::edit_table_structure_at` for every variant (row insert/duplicate/clear/delete/move, column insert/duplicate/clear/delete/move/align) with header-row guards and out-of-range `None`; add unit tests in `src/lib.rs` covering each variant, header guards, selection placement, alignment preservation, and column-comment insert/delete/permute sync; verify with `cargo test --lib table_structure`

## 2. Remove the in-flow header toolbar and dwell state

- [x] 2.1 Delete the header toolbar row, `VISUAL_TABLE_TOOLBAR_ACTIONS`, toolbar button/visibility helpers, the dwell/hide timers (`arm_/fire_visual_table_hover_dwell`, `arm_/fire_visual_table_hide_delay`), their constants and per-tab fields; add `hovered_visual_table_cell` with resets at the existing hover reset sites; remove or rewrite the header-toolbar tests in `src/app/tests.rs`; verify `cargo build` and `cargo test --bin markion visual_table` pass

## 3. Overlay handles, edge strips, and menu

- [x] 3.1 Split `visual_table_view` into an outer relative wrapper and the inner clipped grid; add per-cell hover tracking, row grips (first-cell left padding), column grips (header-cell top padding), and the bottom/right "+" strips as absolute overlays; add app tests asserting grips/strips appear only for the hovered row/column/table and that the table's `debug_bounds` are identical before and after hover; verify with `cargo test --bin markion visual_table_handle`
- [x] 3.2 Add `VisualTableAxisTarget`, `revalidate_visual_table_axis_target`, `visual_table_menu` app state (cleared by `dismiss_visual_block_menu`, Escape, mouse-down-out), the row/column menu view rendered via `anchored()` in `root_view.rs`, per-item enablement and alignment "current" marker, and i18n strings (English + Simplified Chinese) in `src/i18n.rs`; add app tests that open each menu, check disabled header-row items, apply one row and one column action as a single undo step, and verify stale targets are no-ops; verify with `cargo test --bin markion visual_table_menu`
- [x] 3.3 Wire the edge strips to append a row/column through `edit_table_structure_at`; add app tests clicking both strips (one undo step, isolated to one of two tables); verify with `cargo test --bin markion visual_table_edge`

## 4. Drag reorder

- [x] 4.1 Add `DraggedTableRow` / `DraggedTableColumn` drags on the grips (none for the header row), `on_drop` + border-color `drag_over` indicators on rows/cells, applying `MoveRow` / `MoveColumn`; add app tests that call the drop path and check the resulting source, one undo step, and no-op on self-drop; verify with `cargo test --bin markion visual_table_drag`

## 5. Integration

- [x] 5.1 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` (root crate), and `cargo test --bin markion` plus `cargo test --lib`; all pass — fmt and both test suites pass (bin 650, lib 677); `clippy -D warnings` still fails on ~120 pre-existing lints in the root crate and workspace members that predate this change, but no lint falls on lines this change touched
- [x] 5.2 Validate the change with `openspec validate notion-style-visual-table-handles --strict`
