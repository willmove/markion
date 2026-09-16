## Why

Visual Edit tables already size columns from content and allow direct cell text editing, but users still cannot drag column widths, apply Bold/Italic/Code/Link to a selection inside a cell from the block context menu, or drag an inline image to a non-preset width. Those three gaps are the remaining high-frequency table/image precision operations.

## What Changes

- Visual Edit GFM tables expose draggable handles between adjacent columns. Dragging is presentation-only until mouse-up, which writes one undoable HTML comment immediately before the table: `<!-- markion-cols:30,70 -->`. Read/Split use the same widths. Without a comment, content-heuristic weights remain.
- A non-empty selection fully inside one Visual Edit table cell exposes the existing Bold / Italic / Inline Code / Link context-menu group. Keyboard formatting of that cell selection uses the same source wrap path. Cross-cell or `|`-crossing selections stay conservative.
- Focused inline `![alt](url)` images keep the 25/50/75/100% buttons and gain a drag handle. Width metadata accepts integer percents 10–100 (not only presets). Dragging overlays the live size without bumping document version; mouse-up commits one `set_image_presentation_at` mutation. Nearby presets snap.

### Non-goals

Indented code, unclosed fences, multiline/malformed/reference-style images, rewriting reference images to inline form, GFM definition lists, HTML `<table>` colspan drag handles, persisting widths in app settings, changing export column grids, or true WYSIWYG projection of cell markup (focused cells still reveal source).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tables-outline`: Draggable GFM column widths persisted as an absorbed HTML comment; Visual Edit cell selections get the existing inline-format controls when they stay inside one cell.
- `document-resources`: Inline image width is an integer 10–100% (presets remain), and Visual Edit supports drag-resize of exactly mapped inline images.
- `markdown-editing`: Selection-contextual format controls cover a single table cell; a `<!-- markion-cols:… -->` comment immediately before a GFM table is absorbed into that table’s preview/visual source range rather than rendered as Html; coverage matrix notes drag-resize and cell format.

## Impact

- **Code:** `src/table.rs` (comment parse/format, percent redistribution, flex-from-authored); `src/lib.rs` (absorb comments after preview derivation, `edit_table_at` keeps comments in sync on column add/delete, `set_table_column_widths_at`, format gate); `src/visual.rs` (cell ranges skip the comment prefix); `src/inline_edit.rs` (width 10–100); `src/app/preview.rs` (handles, overlay widths); `src/app/editing.rs` (cell format target, drag commit); `src/app/state.rs` / `src/app/mod.rs` / `src/app/root_view.rs` (presentation-only drag overlay, drop on the workspace row); `docs/visual-editing-quality.md`; `README.md` / `README.zh-CN.md`.
- **Invariants:** Drag overlay MUST NOT bump document version or invalidate per-version `Arc` caches. Mouse-up is one canonical `MarkdownDocument.text` mutation. `crates/*` stay GPUI-free.
- **Tests:** Pure helpers for comments/percents/gates; derivation tests that the comment is not an Html island; GPUI tests that overlay is version-stable, mouse-up is one undo step, and a cell selection exposes Bold.
