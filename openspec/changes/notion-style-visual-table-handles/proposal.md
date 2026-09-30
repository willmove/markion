## Why

In Visual Edit, table editing controls live in an in-flow header row that is inserted above the table on hover or when a cell owns the caret. Inserting and removing that row shifts the table and everything below it, so hovering or clicking into a table makes the document visibly jump — the click itself can land on a different line than intended. The hover dwell (`table-toolbar-hover-dwell`) only reduces how often this happens. Mainstream block editors (Notion, Obsidian Live Preview, Milkdown/Tiptap-based editors) never put table controls in the document flow; they use per-row/per-column handles, edge "+" affordances, and menus, all overlaid on the table.

## What Changes

- **BREAKING (UI)**: Remove the Visual Edit table editing header row (`+Row`, `-Row`, `Up`, `Down`, `+Col`, `-Col`, delete-table buttons), along with its hover dwell and hide-delay timers.
- Add Notion-style **row handles**: hovering a row reveals a small grip in that row's leading cell padding. Clicking it opens a row menu: Insert above, Insert below, Move up, Move down, Duplicate, Clear contents, Delete. Actions that would break the GFM header invariant (for example deleting, moving, or inserting above the header row) are shown disabled.
- Add Notion-style **column handles**: hovering any cell of a column reveals a grip in that column's header-cell top padding. Clicking it opens a column menu: Insert left, Insert right, Move left, Move right, Align left / center / right, Duplicate, Clear contents, Delete. Delete is disabled for a single-column table.
- **Drag to reorder**: dragging a row handle onto another body row moves that row, and dragging a column handle onto another column moves that column. Each drop is one undoable source mutation, and the column-width comment is permuted along with the columns.
- **Edge add affordances**: while the pointer is over a table, a thin "+" strip is overlaid just below the table (append a row) and just to its right (append a column).
- All handles, strips, and menus are overlays. Showing or hiding them never changes the table's or the document's layout.
- Whole-table delete, duplicate, and move stay available through the existing block grip and the right-click block context menu.
- Menu labels and status messages go through `src/i18n.rs` (English and Simplified Chinese).

Non-goals: no multi-cell selection or range operations, no sorting or filtering, no header-row toggling, no drag-to-extend "+" strips (one row/column per click), no changes to Split Preview or Read mode, no changes to source-mode table commands or the column-resize handles.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `tables-outline`: the "GFM table rendering with row/column toolbar editing" requirement changes from a hover/caret-revealed header toolbar to overlaid row/column handles with menus, drag reordering, and edge add affordances, all with no layout shift.

## Impact

- Code: `src/model.rs` (new `TableStructureEdit` operation enum), `src/lib.rs` (index-addressed `edit_table_structure_at` sharing the existing format + column-width-comment rewrite path with `edit_table_at`), `src/table.rs` (column-percent permutation helper), `src/app/preview.rs` (`visual_table_view` handles, overlays, drop targets; menu rendering), `src/app/mod.rs` / `src/app/state.rs` (hovered-cell and table-menu state; the dwell state is removed), `src/app/application.rs` (the dwell timers are removed), `src/app/root_view.rs` (menu overlay), `src/app/editing.rs` (applying the edits), `src/i18n.rs`, and `src/app/tests.rs` (the header-toolbar tests are replaced).
- Invariants: every table operation is one source mutation through the existing history/dirty/version path, so the derived visual blocks are rebuilt once per new document version. Hover state and menu state are per-tab or per-app UI state and never bump the document version or invalidate derived Markdown caches.
- Supersedes the in-flight `table-toolbar-hover-dwell` change, whose header toolbar no longer exists. It should be dropped rather than archived.
