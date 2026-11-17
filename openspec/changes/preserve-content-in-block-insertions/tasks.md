## 1. Content-preserving canonical edits

- [x] 1.1 Reject non-empty Divider/Table transformations and make divider insertion below a complete unit or at an empty paragraph create a source-backed blank continuation paragraph. Extend block-edit tests for table column-width comments/cells/alignments, prose/code, neighbors, EOF, LF/CRLF and stale targets; verify with `cargo test -p markion --lib block_edit::tests` while retaining canonical cache/version invariants.

## 2. Compact menu shortcuts and localization

- [x] 2.1 Remove Divider/Table conversions from grip and ordinary editing roots, add scoped localized compact insertion shortcuts immediately after Add Below, and route both through existing checked insertion. Preserve image/table-cell catalogs and the existing compact Insert flyout. Update affected catalog/layout tests and verify focused grip/context-menu tests, including actual pointer/keyboard dispatch, source/selection/history/cache preservation before confirmation, and one-step Undo/Redo after insertion.

## 3. Integration verification

- [x] 3.1 Run `cargo fmt --all -- --check`, focused block-edit, grip, context-menu and visual-block-menu suites, and `cargo test -p markion`; fix regressions caused by this change. Verify strict OpenSpec validation and final diff scope, preserve the pre-existing compact-flyout work, and record available live/manual inspection plus any unrelated or unavailable checks.
## Verification record — 2026-11-17 (Asia/Shanghai)

- `cargo test -p markion --lib block_edit::tests`: 21 passed, including byte-preserving insertion after tables with column-width metadata, LF/CRLF, EOF, empty paragraphs, continuation typing, and rejected template conversion/stale targets.
- `cargo test -p markion`: 1641 passed and 3 existing tests ignored (optional export-engine package checks and two external-network tests); no failures. Includes the grip, context-menu, visual-block-menu, localization and integration suites.
- After preserving the existing image/table-cell operation grouping in the final layout adjustment, `cargo test -p markion --bin markion`: 869 passed, 2 existing network tests ignored; no failures.
- `cargo fmt --all -- --check`, `git diff --check`, and `openspec validate preserve-content-in-block-insertions --strict`: passed.
- Rendered GPUI tests exercise actual table grip/button clicks, compact insertion controls on the same row, both themes, 340x280 keyboard navigation, exact source/structured-selection Undo/Redo and unchanged cache identity before confirmation. A standalone native-window manual inspection was not performed.
- The pre-existing compact Insert flyout work is retained. Stable specs, dependencies and release metadata were not modified; this completed change remains unarchived.

