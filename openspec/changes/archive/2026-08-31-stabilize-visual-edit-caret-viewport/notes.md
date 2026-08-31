# Implementation Notes — stabilize-visual-edit-caret-viewport

## Task 1.3 — caret viewport inset

Chose **one `preview_row_line_height`** (default `PREVIEW_LINE_HEIGHT` = 23px), not a fixed 8–12px band.

Reason: the last-line typing fixture paints a caret whose height is the preview line. A 2px margin (the previous `follow_visual_caret_in_list` value) left the caret flush with the clip, so the next glyph immediately overflowed and forced another follow. One line of inset gives that glyph room without being large enough to pull a mid-pane click.

Call sites pass the live `typography.preview_row_line_height` so a custom rendered font size scales the inset. The helper constant `VISUAL_CARET_VIEWPORT_INSET` is the default-typography value used by unit tests.
