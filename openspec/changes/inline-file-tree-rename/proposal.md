# Proposal: inline-file-tree-rename

## Why

The file-tree rename (and create) flow is effectively unusable: the name prompt is not a real input control but a keystroke-redirected state flag, and four routing defects make the flow fail in practice — any left mouse-down anywhere below the menu bar silently cancels the prompt (`close_menu` clears `pending_name_input`), the editor pane's mouse handlers do not know about the prompt (clicking/dragging moves the document caret instead), arrow/Home/End/Select-All keys are not redirected (they move the source caret), and the prompt's bare-`String` buffer has no cursor, selection, visible caret, or base-name pre-selection. Users report that a rename has essentially never completed, and stray keystrokes after the silent cancellation land in the open document.

## What Changes

- Render the rename/create name editor **inline in the file-tree row** (replacing the row's label while the prompt is open), instead of a detached prompt at the top of the tree panel.
- Give `PendingNameInput` a cursor position and a selection range, with full editing: typed characters replace the selection, Backspace/Delete, Left/Right/Home/End/Ctrl+A (Select-All) operate on the name buffer, and a visible caret plus selection highlight are rendered.
- On open for **Rename**, pre-select the base name and preserve the extension (`report.md` → select `report`); typing replaces the selection in one stroke. Create prompts keep the current full-name prefill behavior with the whole buffer selected.
- Fix event routing while the prompt is open:
  - mouse-down inside the prompt editor and the file-tree panel no longer bubbles to `workspace-row`'s `close_menu`, so the prompt is not silently killed;
  - the editor pane's `on_mouse_down` / `on_mouse_move` become no-ops (or explicitly cancel the prompt) while the prompt is open, so pointer interaction no longer moves the document caret;
  - tree-row clicks while the prompt is open no longer open files.
- Define a deliberate click-away policy: clicking outside the inline editor commits the rename (Explorer-style) when the buffer parses to a valid, changed name; Escape cancels; Enter commits (unchanged).
- Keep the existing rename pipeline untouched: dirty-document refusal, unique-name collision avoidance, tab re-pointing, status messages.

Non-goals: moving entries via drag-and-drop; a modal rename dialog; changing the tab-bar Rename pipeline (it reuses this prompt and inherits the fixes); multi-line or IME-specific behavior beyond what the redirected-text-input path already handles.

## Capabilities

### New Capabilities

- `inline-name-editing`: The inline, in-row name editor used by file-tree rename/create and the tab-bar Rename action — its rendering position, editing model (cursor, selection, base-name pre-selection), and its input-routing contract (which mouse and key events it owns, click-away commit vs cancel, Escape/Enter semantics).

### Modified Capabilities

- `workspace`: The "Create, rename, delete, refresh" tree operations and the "Tab rename reuses the file rename pipeline" requirement are updated to reference the inline in-row editor and its routing/click-away contract instead of the detached "inline name prompt".

## Impact

- `src/app/mod.rs` — `PendingNameInput` gains `cursor: usize` / `selection: Range<usize>` (or equivalent), possibly a `row_path` for in-row rendering.
- `src/app/root_view.rs` — `pending_name_prompt_view` moves into the tree row (`file_tree_panel_body`); tree-row mouse handlers gain prompt-aware guards; prompt editor renders caret/selection and does its own hit-testing; the under-tab-bar fallback rendering stays for the hidden-sidebar case.
- `src/app/editing.rs` — `close_menu` no longer clears `pending_name_input`; `on_mouse_down`/`on_mouse_move` guard on prompt state; `left/right/home/end/select_all/backspace` gain `has_text_input_focus()` branches operating on the name buffer.
- `src/app/application.rs` — `active_input_text_mut` / `insert_redirected_text` / `pop_text_input` updated to respect the buffer's cursor/selection (replace-on-insert, delete-selection).
- `src/app/workspace.rs` — `open_name_prompt` computes the base-name selection; tree-row open-on-click guard; `confirm_pending_name` unchanged in substance.
- `src/i18n.rs` — any new user-facing strings (e.g. click-away commit feedback) must go through the i18n table.
- Tests in `src/app/tests.rs` — existing `PendingNameInput` tests updated for the new fields; new tests for routing fixes (mouse-down does not cancel; editor caret does not move while prompt open; arrow keys edit the buffer; click-away commits).
- Invariants touched: the file tree still renders a bounded number of rows per frame (the inline editor renders within the row, not as an overlay over the whole panel); the redirected-text-input path remains the single IME entry point for the buffer.
