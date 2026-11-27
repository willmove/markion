## 1. Core helpers, icon, and localization

- [x] 1.1 Add `equal_column_percents` in `src/table.rs`, `can_set_table_column_widths_at` in `src/lib.rs`, and `insert_content_below` in `src/block_edit.rs` with unit tests.
- [x] 1.2 Add `ContextMenuMsg::EqualizeColumnWidths` across all languages in `src/i18n.rs` (including parity tests) and add the `columns` SVG icon in `assets/icons/ui/columns.svg` and `src/ui/icon.rs`.

## 2. Table grip menu and Insert Below unification

- [x] 2.1 Update `BlockMenuState`, `BlockMenuItem`, `BlockMenuPresentation`, and `src/app/preview.rs` to add Equalize Column Widths to the table grip root, remove root `AddBelow` on table grips, and unify `BlockContextSubmenu::Insertion` with the compact 21-item catalog.
- [x] 2.2 Implement Equalize Column Widths execution and deferred below-block insertion (`block_below`) for Link, Formula, and Image editors (including remote URL and local file image insertion) in `src/app/editing.rs` and `src/app/image_operations.rs`.

## 3. Regression verification

- [x] 3.1 Update existing table grip tests and add rendered pointer/keyboard tests for Equalize Column Widths and the expanded Insert Below flyout (blocks, callouts, Link/Formula/Image confirm & cancel, stale targets, LF/CRLF, undo/redo, cache preservation).
- [x] 3.2 Run `cargo test --workspace` and verify all suites pass.
