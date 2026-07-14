## 1. Preview Table Rendering

- [x] 1.1 Remove the complete table editing header and its row/column button wiring from the `PreviewBlock::Table` rendering branch used by Split Preview and Read mode, while leaving the grid and selectable cell text intact.
- [x] 1.2 Verify `visual_table_view`, the shared table-button helper, and all source table commands remain wired to the existing source-backed mutation path.

## 2. Regression Coverage

- [x] 2.1 Add or update focused coverage that distinguishes toolbar-free preview table rendering from toolbar-enabled Visual Edit table rendering.
- [x] 2.2 Run targeted table, preview-selection, and view-mode tests, then run `cargo test` for the root package.
- [x] 2.3 Manually verify a GFM table has no editing header in Split Preview or Read mode, retains selectable cell text, and still exposes working table controls in Visual Edit.

## 3. OpenSpec Verification

- [x] 3.1 Run `openspec validate remove-preview-table-toolbar` and confirm the change remains apply-ready.
