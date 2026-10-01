## Context

Visual Edit renders each proven GFM table cell from `VisualBlockEditor::Table.cells`. Every cell already carries a logical `(row, column)` and an exact authored `source_range`, while the active tab's canonical selection identifies the current caret endpoint. The shared document mutation path, `MarkdownDocument::edit_table_at`, converts an offset into a `TablePosition`, rewrites the source table once, and returns the post-edit table and cell ranges.

The toolbar currently ignores that metadata. `visual_table_view` passes only `block.source_range.start` to `preview_table_button`; on mouse-up the handler first resets `selected_range` to that table-start offset and then calls `apply_table_edit_at`. `table_position_at` consequently resolves row 0, column 0 for every toolbar command. Header deletion and movement are rejected as invalid, and the remaining commands use fixed row/column positions. Existing tests prove only that six actions are listed in Visual Edit and that `edit_table_at` works when called with a real cell offset; no test crosses the toolbar event boundary.

The fix must keep `MarkdownDocument.text` as the only editable representation, reuse the existing table parser/formatter, preserve one undoable source replacement, and invalidate the versioned derived state only through the normal `after_document_changed` path.

### Data flow

```text
cell pointer/keyboard focus
  -> canonical selected_range and caret endpoint
  -> current VisualBlockEditor::Table cell metadata
  -> per-action target + availability for this table
  -> toolbar activation revalidates block/version/cell ownership
  -> apply_table_edit_at(exact cell offset)
  -> edit_table_at -> one formatted source replacement
  -> TableEditResult.selected_range
  -> one undo snapshot + normal version/cache invalidation + grid re-render
```

## Goals / Non-Goals

**Goals:**

- Make every Visual Edit row/column toolbar action operate relative to the cell containing the canonical caret endpoint in that same table.
- Make add, delete, and move placement deterministic for header, body, first/last row, and final-column boundaries.
- Keep the post-edit selection in the expected logical cell and make each successful action one undoable document mutation.
- Render actions without a valid target as visibly and interactively disabled rather than dispatching a misleading no-op.
- Use one compact set of button metrics for all six actions and both availability states.
- Cover the UI-to-document boundary as well as the existing pure table algorithm.

**Non-Goals:**

- Structurally redesigning labels, the action set, or layout, or introducing drag handles.
- Deleting or moving the header row, or moving columns.
- Changing source-mode menu/shortcut command targeting.
- Adding a second table parser, mutable rich-table model, or member-crate GPUI dependency.

## Decisions

### 1. Resolve the target from current visual cell metadata

Derive a small toolbar context from the current document version, the table's stable `VisualBlockId`, the canonical caret endpoint, and `VisualBlockEditor::Table.cells`. A cell owns the caret when its exact field range contains the endpoint, including the valid end position used by empty cells and end-of-cell carets. The context records the logical row/column and a UTF-8-safe source offset inside that field.

Each control receives an optional validated target. If the caret belongs to another block or another table, this table has no target and all six controls are disabled. The handler MUST NOT fall back to `block.source_range.start`.

At activation, revalidate the captured document version, block identity, and current cell ownership before mutation. This prevents a delayed mouse-up or intervening edit from applying an old byte offset to a newly formatted table.

Alternatives considered:

- Use `app.cursor_offset()` directly without checking table ownership. This fixes the common case but could let a toolbar mutate the table containing an unrelated or stale offset.
- Store a separate "last active table cell" in tab state. The canonical selection and existing cell metadata already express ownership; another state variable could drift after edits, undo, or mode changes.
- Keep using the table start and pass row/column separately. This expands the document API and duplicates target information already proven by the visual cell field.

### 2. Define action placement and availability centrally

A pure availability helper will use the active `(row, column)` and current table dimensions so rendering and dispatch share the same rules.

| Action | Mutation target/result | Available when |
|---|---|---|
| Add row | Insert immediately after the active row; from the header, create the first body row; select the new row at the active column | An active cell exists |
| Delete row | Delete the active body row; select the next surviving body row or the previous row when the deleted row was last, at the nearest valid active column | Active row is a body row |
| Move row up | Swap the active body row with the preceding body row; keep the active column in the moved row | Active row is below the first body row |
| Move row down | Swap the active body row with the following body row; keep the active column in the moved row | Active row is a non-final body row |
| Add column | Insert immediately after the active column; select the inserted column in the active row | An active cell exists |
| Delete column | Delete the active column; select the column now at that index or the preceding final column, in the active row | More than one column exists |

The add-row implementation must remove the existing `selected_row.max(1)` behavior that turns a header target into insertion after the first body row. It must also preserve the active column rather than resetting it to column zero.

Alternatives considered:

- Allow a disabled row command to run and report `StatusNoTableAtCursor`. That status is inaccurate because a table exists; a disabled control communicates the structural boundary before activation.
- Choose the first body row or first column when no cell owns the caret. This recreates the surprising fixed-position behavior the change is intended to remove.

### 3. Retain the shared source-table mutation path

Successful controls continue through `apply_table_edit_at` and `MarkdownDocument::edit_table_at`. `src/table.rs` remains the owner of range lookup, parse, normalize, format, and formatted-cell range derivation. The UI supplies an exact target; it does not manipulate row vectors or Markdown text.

`TableEditResult.selected_range` remains the single post-edit selection contract. Row edits preserve the originating column where possible; column edits preserve the originating row. The app installs that returned range, clears reversed/marked selection state, commits exactly one undo snapshot, and calls `after_document_changed` exactly once. Undo and redo therefore restore the complete source table and selection through existing tab history.

No extra preview parse is introduced for toolbar state. Availability is computed from the already cached `VisualBlock` rows and cell metadata for the current document version. The successful replacement triggers the same version bump and derived-cache invalidation as direct cell editing; an unavailable or stale action changes neither text nor version.

### 4. Make disabled controls explicit and non-mutating

`preview_table_button` (or a Visual Edit-specific replacement) will accept enabled/target state. Disabled controls use muted styling, do not use the pointing cursor, and do not install a mutation listener. Enabled controls stop pointer propagation as needed so clicking toolbar chrome cannot first move the canonical caret to the table container.

All six controls share compact metrics of 6 px horizontal padding, 2 px vertical padding, and 10 px text. Enabled and disabled controls use the same geometry so availability changes do not shift the toolbar.

No new user-facing string is required for the initial fix. If implementation adds a tooltip or status message, it must be represented in `src/i18n.rs` for every supported locale.

### 5. Test each ownership layer

- Pure helpers: active-cell ownership for collapsed/reversed selections, empty cells, range ends, UTF-8 content, no active cell, and a cursor in a different table; availability at every structural boundary.
- Document model: every action from non-first rows/columns, header add-row insertion, row-operation column preservation, formatted selection ranges, alignments, escaped pipes, and UTF-8.
- Rendered GPUI interaction: click each enabled toolbar control after focusing a chosen cell; assert exact source, selected logical cell, dirty/version change, one-step undo/redo, disabled controls, and isolation between two tables.
- Regression gate: retain preview/read toolbar absence and source-command behavior, then run the repository quality commands.

## Risks / Trade-offs

- **[Caret ownership is ambiguous at an empty-cell or range-end boundary]** -> Use the already proven `VisualEditorField` ranges and deterministic field order, and add empty/UTF-8/end-boundary tests instead of re-parsing delimiters in the UI.
- **[A render-time byte offset becomes stale before mouse-up]** -> Capture version/block/cell identity and revalidate against current state before dispatch; stale controls perform no mutation and request a fresh render.
- **[Disabled controls look enabled or still receive pointer events]** -> Centralize enabled styling and listener installation in the button helper and assert both presentation state and unchanged document state in rendered tests.
- **[Full-table formatting moves the selection unexpectedly]** -> Continue using `formatted_table_cell_range` through `TableEditResult` and explicitly test each operation's logical row/column after reflow.
- **[Availability calculation adds render work]** -> Scan only the already materialized cells of the current table; do not parse Markdown or invalidate cached derived state.

## Migration Plan

No persisted-data or schema migration is required. Implement the target/availability helpers and tests first, wire the six controls to the exact offset, then adjust add-row selection semantics and run the full gate. The change is rollback-safe because it does not alter stored Markdown syntax or external APIs; reverting restores the old toolbar adapter without requiring document conversion.

## Open Questions

None. Header/body and boundary semantics are fixed by this design and the delta specification.
