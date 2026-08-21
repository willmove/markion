# Design: inline-file-tree-rename

## Context

The current name prompt is a redirected-text-input pseudo-field: `PendingNameInput { kind, parent, target, buffer: String }` (src/app/mod.rs) whose keystrokes are intercepted by checks in action handlers (`insert_newline`, `backspace`, `replace_text_in_range` → `active_input_text_mut`), while rendering is a static label at the top of the tree panel (`pending_name_prompt_view`, src/app/root_view.rs). See proposal.md — Why for the four routing defects. The design below keeps the redirected-text-input path (it already handles IME composition via `input_marked_len` and is shared with the search and filter fields) and upgrades it with an editing model plus in-row rendering, rather than adopting a separate text-input widget.

## Goals / Non-Goals

Goals:

- The name editor behaves like a real single-line input: caret, selection, base-name pre-selection, click-to-position.
- While it is open, no pointer or key event reaches the document editor or triggers tree-row open.
- The file tree keeps its bounded per-frame row budget and the per-keystroke rebuild cost stays flat.

Non-goals: drag-to-move entries; a modal dialog; changing the dirty-refusal / collision / tab-repoint pipeline; touching IME handling beyond keeping it on the redirected path.

## Decisions

### D1 — Editing model on `PendingNameInput`, not a new widget

Add `cursor: usize` (byte offset into `buffer`) and `selection: Range<usize>` (byte offsets; empty = collapsed caret) to `PendingNameInput`. All redirected-key branches operate on these fields:

- `insert_redirected_text` replaces `selection` with the incoming text and collapses the caret after it (adjusting for `input_marked_len` composition exactly as today).
- `pop_text_input` (Backspace) deletes the selection, or one char before the caret when collapsed.
- New `left/right/home/end/select_all` branches in the existing action handlers (`has_text_input_focus()` guard) move/extend the caret within the buffer; Ctrl+A selects the whole buffer.

Alternatives considered: adopting a GPUI text-input crate element. Rejected — it would fork the IME path the app already relies on, and the document editor itself is hand-rolled on the same redirected pattern; consistency wins.

### D2 — In-row rendering replaces the panel-top prompt

`file_tree_panel_body` renders each row via `file_tree_row_view`. When `pending_name_input` targets that row's path (rename) or the row is the new-entry row (create), the row's label child is replaced by the editor view: same row height, the buffer text with a caret quad and a selection-highlight quad painted via the `fill()` idiom used by `EditorElement` for its caret. Hit-testing inside the row reuses `estimate_file_tree_text_width` (already used for `file_tree_content_width`) to map a click x-coordinate to a byte offset in the buffer (character granularity, left-edge snapping like a normal text field).

The existing under-tab-bar fallback (`root_view.rs` "document-workspace-column" `.when(...)` branch) stays for tab-bar Rename with the sidebar hidden, upgraded to the same caret/selection rendering. `pending_name_prompt_view` at the tree-panel top is removed.

For create actions the prompt currently has no row. Render a synthetic row (with the new name's icon) at the top of the target folder's children — the `filtered_visible_file_tree_entries` iterator gains an insertion point driven by `pending.parent` + `pending.kind`, respecting the 300-row cap (the editor row itself is always rendered, even when the cap is hit, by reserving one slot).

### D3 — Event routing fixes (the bug fixes)

- `close_menu` (editing.rs): remove `pending_name_input` from its kill list; it stays for menus only. The name editor's lifecycle is governed by D4's click-away commit.
- Editor pane: in `on_mouse_down`, if `pending_name_input.is_some()`, run the click-away commit (D4) and return before `move_to`/`is_selecting`. `on_mouse_move`/`on_mouse_up` are then naturally inert because `is_selecting` never started.
- Tree rows: their `on_mouse_up(Left)` handlers check `pending_name_input.is_some()` first → run click-away commit instead of opening the file. Right-click on another row already replaces the prompt via `show_file_tree_context_menu` (keep, it is an explicit action).
- The row editor view itself registers `on_mouse_down(Left)` with `cx.stop_propagation()` so the click never bubbles to `workspace-row`'s `close_menu`, then positions the caret (Shift extends the selection).
- The under-tab-bar fallback registers the same stop-propagation guard.

### D4 — Click-away commits (Explorer semantics)

A left mouse-down anywhere outside the field while the editor is open resolves it: if the trimmed buffer is non-empty, unchanged in name, or refused (empty name, dirty active document, collision) the editor stays open and the status bar shows the localized refusal message; otherwise commit via the existing `confirm_pending_name` path. Escape still cancels outright.

Rationale: cancel-on-click-away (current de-facto behavior) is what destroys the user's typed name today; commit-on-click-away matches Explorer/Finder and makes "click elsewhere after typing" do the expected thing. Alternative considered: keep cancel-on-click-away but make it explicit and animated — rejected as hostile to the primary rename flow.

### D5 — Pre-selection policy

`open_name_prompt` computes the initial selection: for `PendingNameKind::Rename`, select the base name (up to the last `.` in the final path component, when that dot is not the first character); otherwise select the whole prefilled buffer. `confirm_pending_name` is unchanged — it already reads `pending.buffer`.

## Risks / Trade-offs

- [Click-away commit fires when the user meant to cancel] → Escape is the documented cancel path; the click-away commit only reuses the exact Enter pipeline including refusals, so a refused commit leaves the editor open.
- [Byte-offset cursor vs multi-byte CJK names] → caret movement uses `char_indices` boundaries (the document editor already does byte-offset-with-char-boundary checks; reuse the same helpers); hit-testing rounds to char boundaries.
- [In-row editor increases per-keystroke tree rebuild cost] → the tree already rebuilds per keystroke when the filter is focused and caps rows at 300; the editor row adds O(1) work. The prompt deliberately does not run tree filtering (existing behavior).
- [IME composition while selection non-empty] → `insert_redirected_text` already strips `input_marked_len` before replacing; the new selection-replace path must do the same — covered by a dedicated test.
- [Tab-bar fallback click-away while sidebar hidden] → the same commit handler runs from the fallback field's context; document clicks route through the editor-pane guard identically.

## Migration Plan

Pure UI-layer change, no persistence or schema impact; single change set, no staged rollout. Rollback = revert the change commit; the rename pipeline itself is untouched.

## Open Questions

- Whether the create-file editor row should appear above or below the target folder row when the folder is collapsed (currently the prompt renders at the panel top regardless). Safe to decide during implementation without changing the specs: both satisfy "in place".
