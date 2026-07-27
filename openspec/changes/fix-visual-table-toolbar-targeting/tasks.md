## 1. Target and Availability Model

- [x] 1.1 Add a pure Visual Edit table-toolbar context helper that resolves the canonical caret endpoint to an exact `VisualBlockEditor::Table` cell in the same `VisualBlockId`, records the current document version and UTF-8-safe source offset, and returns no target for another block/table instead of falling back to the table start.
- [x] 1.2 Add a pure per-action availability helper for header/body, first/last body row, and final-column boundaries, and unit-test collapsed and reversed selections, empty cells, end-of-cell carets, UTF-8 cells, stale versions, and two-table isolation.

## 2. Shared Table Edit Semantics

- [x] 2.1 Update `MarkdownDocument::edit_table_at` row insertion so an active header inserts the first body row, a body target inserts immediately after itself, and the returned selection preserves the active logical column instead of resetting to column zero.
- [x] 2.2 Extend document-model tests to cover all six edits from non-first rows/columns, header add-row placement, first/last-row rejection, final-column rejection, post-format logical row/column and exact selection ranges, alignments, escaped pipes, and UTF-8 content.

## 3. Visual Edit Toolbar Wiring

- [x] 3.1 Compute each visible Visual Edit table's toolbar target and action availability from its cached visual cell metadata and the canonical caret without re-parsing Markdown or changing versioned derived caches.
- [x] 3.2 Replace the `table_offset` selection reset in the toolbar handler with exact-cell dispatch through `apply_table_edit_at`; revalidate document version, block identity, and cell ownership at activation and keep stale or cross-table actions non-mutating.
- [x] 3.3 Render unavailable controls with consistent disabled styling and no mutation listener or pointing cursor, while enabled controls preserve pointer propagation/focus so the cell target is not changed before dispatch.
- [x] 3.4 Preserve the existing single snapshot, dirty-state, selection restoration, `after_document_changed`, and localized-status paths for every successful toolbar action; add localized strings in `src/i18n.rs` only if the implementation introduces new visible feedback.
- [x] 3.5 Apply shared compact metrics of 6 px horizontal padding, 2 px vertical padding, and 10 px text to all six toolbar controls in both enabled and disabled states without changing action targeting.

## 4. Interaction Regression Coverage

- [x] 4.1 Add rendered GPUI tests that focus a chosen table cell and activate each enabled row/column control, asserting exact Markdown output, selected logical cell, one document-version change, dirty state, and grid re-rendering.
- [x] 4.2 Add rendered tests proving header/first/last/final-column controls and toolbars without an owned caret are disabled and leave source, selection, version, dirty state, and undo history unchanged.
- [x] 4.3 Add two-table and undo/redo tests proving only the caret-owning table can be targeted and each successful toolbar operation is reverted and reapplied in one history step.
- [x] 4.4 Retain regression coverage that Split Preview and Read mode expose no table controls and that source-mode table commands continue to target the source caret through the existing command path.
- [x] 4.5 Add regression coverage for the shared compact toolbar metrics and rerun all six rendered toolbar interactions to ensure the smaller controls remain functional.

## 5. Documentation and Verification

- [x] 5.1 Update the GFM-table row in `docs/visual-editing-quality.md` so required evidence explicitly includes toolbar cell ownership, structural availability, multi-table isolation, and one-edit history behavior.
- [x] 5.2 Run `cargo fmt --all --check`, focused table/toolbar tests, `cargo test`, and `cargo test --workspace`; resolve failures without introducing GPUI dependencies into workspace member crates.
- [x] 5.3 Run `openspec validate fix-visual-table-toolbar-targeting` and confirm the change remains apply-ready with every completed implementation task checked off.
- [x] 5.4 Run formatting checks, focused table-toolbar tests, the root test suite, and OpenSpec validation after the compact styling change.
