## Why

Visual Edit's table toolbar discards the active cell before dispatching an edit and always supplies the table's first source offset. The document model therefore resolves every toolbar action to the header's first cell: delete/move-row actions become silent no-ops, while add-row and column actions operate at fixed positions instead of the cell containing the caret.

## What Changes

- Preserve the active Visual Edit table-cell context when a toolbar control is activated and route the edit to that cell in the toolbar's own source table.
- Define precise toolbar semantics: row operations target the active body row, column operations target the active column, and additions are placed immediately after that target.
- Keep the resulting canonical selection in the logical cell returned by the table edit so the re-rendered grid retains a meaningful editing position.
- Represent unavailable operations explicitly instead of presenting a clickable control that silently does nothing, including header row deletion/movement, table-boundary movement, and deletion of the final column.
- Compact all six table toolbar controls with smaller shared padding and text while keeping enabled and disabled geometry consistent.
- Add pure document, toolbar-targeting, rendered interaction, undo/redo, UTF-8, and multi-table regression coverage for every row and column control.
- Update the Visual Edit quality matrix's table evidence to cover toolbar target ownership and invalid-operation behavior.

Non-goals: structurally redesigning the table toolbar or its action set, adding drag-and-drop reordering, allowing the header row to be deleted or moved, changing source-mode table command semantics, or introducing a second table parser or editable document model.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tables-outline`: Strengthen Visual Edit toolbar requirements so operations target the cell containing the canonical caret in the toolbar's table, preserve post-edit selection, and expose unavailable operations without silent no-ops.

## Impact

- Visual Edit table rendering and pointer handlers in `src/app/preview.rs`.
- Table edit dispatch, status handling, undo capture, selection restoration, and document-change invalidation in `src/app/editing.rs`.
- Shared table targeting/mutation helpers in `src/lib.rs` and `src/table.rs` only if the UI adapter cannot express an exact target through the existing `edit_table_at` contract.
- Focused root-package GPUI and document-model tests, plus `docs/visual-editing-quality.md`.
- No dependency, storage-format, export-format, or public API changes are expected.
- The change preserves the canonical-source, single-history-mutation, exact UTF-8 range, and per-document-version derived-cache invariants.
