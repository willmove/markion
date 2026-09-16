## Context

See proposal.md for motivation. GFM tables already use `table_column_flex_weights` as presentation-only `flex_grow`. Visual table cells edit source through `VisualBlockEditor::Table` (empty `editable_runs`), so `visual_selection_format_target_for_block` currently returns `None` and the block context menu omits Bold/Italic/Code/Link. Inline image width metadata only accepts `25|50|75|100` and the focused chrome is preset buttons.

Cached-per-version Markdown must stay intact: pointer-move during a drag is tab-local overlay, not a document edit.

## Goals / Non-Goals

**Goals:**

- Persist user column widths in canonical Markdown as a single HTML comment line immediately before the pipe table, absorbed into the table preview/visual block so it is not an Html island.
- One undoable mutation per completed drag (columns or image width).
- Cell format controls when the selection is fully inside one `VisualEditorFieldKind::TableCell`.
- Inline image width 10–100 with drag overlay.

**Non-Goals:**

- Measuring live GPUI glyph runs for the heuristic fallback.
- Drag handles on HTML `<table>` grids or one-column GFM tables that have no cell editor.
- Inventing a comment when the user only uses Add Column (comment is created on the first successful column-width drag, or rewritten when it already exists).

## Decisions

### Column widths live in `<!-- markion-cols:N,N,… -->`

GFM has no column-width syntax. An HTML comment is ignored by CommonMark consumers and stays valid Markdown. Alternative considered: pipe-padding in the source table — rejected because Format/toolbar already reflows padding and would fight user widths.

After `derive_preview_and_outline` sorts blocks, if an Html block’s trimmed payload parses as this comment and the next block is a Table, extend the table `source_range` start to the comment and remove the Html block (recurse into quote children). Read/Split therefore never paint the comment. Whole-table delete/duplicate use the absorbed range.

`table_range_at` still returns only pipe lines so cell edits and toolbar offsets stay on the GFM table. `visual_block_editor` skips the leading comment prefix before `table_cell_source_ranges`.

### Overlay during drag; commit on drop

`DocumentTabState` holds `visual_table_column_drag` / `visual_image_resize_drag`. `on_drag_move` updates live percents and `cx.notify()` without `after_document_changed`. `on_drop` on the workspace row (same pattern as the sidebar handle) commits even if the pointer is no longer over the table.

Adjacent-column drag: left/right share their current percent sum; each stays at least `max(1, min(5, floor(100/n)))`. Other columns unchanged.

Without an existing comment, the overlay starts from `percents_from_flex_weights` of the current heuristic so the first drag feels continuous.

Add/Delete column: if a comment already exists and its length matches the old column count, rewrite it in the same `edit_table_at` mutation (insert: give the new column the minimum share taken from its left neighbor; delete: fold the removed share into the left neighbor, or right if index 0). If counts mismatch, drop the comment and fall back to the heuristic.

### Cell format target is the table cell field, not `editable_runs`

Tables have a dedicated editor and empty runs. `visual_selection_format_target_for_block` succeeds when the non-empty selection is contained in one `VisualTableCell.field.source_range`. Keyboard `apply_markdown_format` additionally no-ops when the selection starts inside a GFM table but is not contained in one cell, or when the format is not Bold/Italic/InlineCode/Link.

### Image width is any integer 10–100

`split_presentation_title` / `compose_title` accept and emit `width=` in that range. Preset buttons stay. Drag handle is shown only when `inline_image_at` succeeds (inline destinations). Live percent snaps to 25/50/75/100 when within 3 points. Reference-style images stay without the handle.

## Risks / Trade-offs

- **[HTML comment is Markion-specific]** → Other CommonMark tools ignore it; widths degrade to heuristic. Documented in READMEs as optional presentation metadata, same family as `{width=N align=…}` on images.
- **[Drop requires a hovered hitbox]** → Workspace-row `on_drop` covers the editing chrome, matching the sidebar resize handle.
- **[Many-column min share]** → If `n * 5 > 100`, the per-column floor drops to 1 so percents can still sum to 100.
- **[Focused cell still shows source]** → Format wraps `**` around the selected cell bytes; this is format-controls-in-cell, not a second rich-text model.

## Migration

None. Existing tables without the comment keep heuristic widths. Existing `{width=25|50|75|100}` images keep working; other integers in 10–100 become valid.
