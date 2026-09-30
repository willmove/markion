## Context

`visual_table_view` (`src/app/preview.rs`) renders a Visual Edit table as a bordered, rounded `div` with `overflow_hidden()`. Its first child is the editing header row, shown when the pointer has dwelled on the table or the caret owns a cell. Header buttons act on the **caret's** cell through `MarkdownDocument::edit_table_at(offset, TableEdit)`, and that function also keeps the `<!-- markion-cols:… -->` comment in sync on column insert/delete. Visual blocks already have a block grip, absolutely positioned at `left: -14px`, and a right-click block menu (Duplicate / Move / Delete). The block-menu state lives on `MarkionApp` and is dismissed from ~27 call sites through `dismiss_visual_block_menu()`.

## Goals / Non-Goals

**Goals:** handles and menus that never participate in layout; structural edits addressed by explicit row/column index rather than the caret; one mutation per action; reuse the existing edit, history, and column-comment paths.

**Non-Goals:** keyboard navigation inside the new row/column menu (Escape and outside-click dismissal only); touch or long-press behavior.

## Decisions

### 1. Index-addressed model operation
Add `TableStructureEdit` to `src/model.rs`:
- `InsertRow { at }`, `DuplicateRow(r)`, `ClearRow(r)`, `DeleteRow(r)`, `MoveRow { from, to }`
- `InsertColumn { at }`, `DuplicateColumn(c)`, `ClearColumn(c)`, `DeleteColumn(c)`, `MoveColumn { from, to }`
- `AlignColumn { column, alignment }`

Add `MarkdownDocument::edit_table_structure_at(table_offset, edit) -> Option<TableEditResult>`. The tail of `edit_table_at` (normalize → format → rewrite the matching column comment → `apply_current_range(MutationOrigin::TableEdit, …)`) is extracted into a private helper that both functions call. The helper takes an optional percent transform (insert / delete / permute), so comment handling stays in one place. A pure `permute_column_percents(percents, from, to)` joins the other percent helpers in `src/table.rs`. Operations that would break the header invariant or are out of range return `None` and leave the document untouched.

*Alternative considered:* move the caret to the target cell and reuse `TableEdit`. Rejected: it mutates the selection as a side effect, can't express "insert above / left", alignment, clear, or duplicate, and couples structural targets to caret ownership (which the old toolbar needed and the handles don't).

### 2. Handles live inside cell padding
Cells have 8 px padding (`p_2`). Each row's first cell gets an absolutely positioned 8 px-wide grip in its left padding, vertically centered. Each header cell gets an 8 px-tall grip in its top padding, horizontally centered. This avoids:
- clipping by the table's `overflow_hidden()` (kept for rounded corners);
- collision with the block grip at `left: -14px`;
- any change to the layout of cell text.

Handles are conditionally rendered absolute children, so their presence never affects layout.

*Alternative considered:* straddle the table border like Notion. Rejected: it requires removing `overflow_hidden` and re-rounding the edge cells, and the header-row grip would overlap the block grip.

### 3. Edge strips live on an outer wrapper
The table splits into an outer `relative` wrapper (id, hover listener, `mb_3`, column-drag listener) and the inner bordered/clipped grid. The two "+" strips are absolute children of the outer wrapper. The bottom strip (`top: 100%`, 10 px) sits in the 12 px bottom margin. The right strip (`left: 100%`, 10 px) sits in the pane's right padding. Neither affects layout.

### 4. Hover state
`hovered_visual_table_block` stays; it drives the edge strips. A new per-tab `hovered_visual_table_cell: Option<(VisualBlockId, row, column)>` is set by an `on_hover` listener on each cell and cleared when the pointer leaves the table. Both are reset at the existing reset sites (tab switch, mode change, document replace). The dwell fields `visual_table_toolbar_hover_ready` and `visual_table_hover_generation`, the two timers, and their constants are deleted, because nothing shifts layout any more and there is nothing to debounce. Hover changes only call `cx.notify()`; they never touch the document, so per-version derived caches are untouched.

### 5. Menu state and dismissal
`MarkionApp.visual_table_menu: Option<VisualTableMenuState { target: VisualTableAxisTarget, anchor }>`. `VisualTableAxisTarget { document_version, block_id, axis: Row(r) | Column(c), row_count, column_count }` is captured at render time. `dismiss_visual_block_menu()` also clears `visual_table_menu`, so every existing dismissal site (caret moves, mode/tab switches, scroll wheel, Escape) covers the new menu without new call sites. Opening either menu closes the other. The panel also closes on `on_mouse_down_out`. It renders through `anchored()` in `root_view.rs`, next to the block menu, and reuses the block menu's visual metrics.

### 6. Revalidation
Every action (menu item, drop, edge strip) calls `revalidate_visual_table_axis_target(target, version, blocks)`. It requires the same document version, a block with the same id that is still a table with an editor, and identical row and column counts. It returns the table's pipe-source start offset. On failure the action is a no-op apart from `cx.notify()`.

### 7. Drag reorder
The row grip calls `.on_drag(DraggedTableRow { target })`, and the column grip calls `.on_drag(DraggedTableColumn { target })`. Every row element has `on_drop::<DraggedTableRow>`, and every cell has `on_drop::<DraggedTableColumn>`. `drag_over` draws a 2 px accent line on the leading edge (moving up/left) or trailing edge (moving down/right), as a border color on an existing border side, so it adds no size. GPUI clears the click state once a drag starts, so `on_click` (open menu) and `on_drag` coexist on the same grip. The header row grip has no `on_drag`.

## Risks / Trade-offs

- [8 px grips are small targets] → The grips use the full padding width/height as their hit area, show a hover background, and are only one of several entry points (the edge strips handle the most common "add" actions).
- [Right strip can be clipped by narrow pane padding] → It is 10 px wide inside ≥ 9 px inner padding plus the reserved scrollbar width. If clipped, the column menu's Insert right is still available.
- [In-flight `table-toolbar-hover-dwell` and `fix-visual-table-toolbar-targeting` deltas modify the same requirement] → This change's delta restates the full requirement. The dwell change is superseded and should be dropped rather than archived after this one.

## Migration Plan

UI-only change with no persisted format change. Rollback is a code revert.
