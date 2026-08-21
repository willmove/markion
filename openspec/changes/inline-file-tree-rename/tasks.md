# Tasks: inline-file-tree-rename

## 1. Editing model

- [x] 1.1 Add `cursor: usize` and `selection: Range<usize>` fields to `PendingNameInput` (src/app/mod.rs); update every constructor call site (workspace.rs, tests.rs) with collapsed-caret defaults
- [x] 1.2 Update `insert_redirected_text` / `active_input_text_mut` (src/app/application.rs) so character input replaces the selection and collapses the caret after the inserted text, preserving `input_marked_len` IME composition semantics
- [x] 1.3 Update `pop_text_input` (Backspace) to delete the selection, or one char before the caret when collapsed; add a Delete-forward variant if the Delete key binding reaches the document today
- [x] 1.4 Add `has_text_input_focus()` branches to `left`, `right`, `home`, `end`, `select_all` (src/app/editing.rs) that move/extend the buffer caret and selection using char-boundary-safe arithmetic, never touching the document
- [x] 1.5 Unit tests in src/app/tests.rs: selection replace, backspace collapsed vs selected, CJK byte-boundary caret moves, IME composition over a non-empty selection

## 2. Event routing fixes

- [x] 2.1 Remove `pending_name_input` from `close_menu`'s dismissal list (src/app/editing.rs); keep menu dismissal behavior otherwise identical
- [x] 2.2 In editor-pane `on_mouse_down` (src/app/editing.rs): when `pending_name_input.is_some()`, run the click-away commit (task 3.3) and return before `move_to` / `is_selecting`; add a test that the document caret and selection are unchanged after a click-drag in the editor pane with the editor open
- [x] 2.3 In file-tree row `on_mouse_up(Left)` (src/app/root_view.rs): when `pending_name_input.is_some()`, run click-away commit instead of opening the file or toggling a folder; add a test that clicking another row with the editor open does not open that file
- [x] 2.4 Regression test: left mouse-down inside the name editor (and in the tree panel around it) does not dismiss the editor; idle mouse movement does not dismiss it

## 3. In-row rendering

- [x] 3.1 Split the tree-row builder in `file_tree_panel_body` (src/app/root_view.rs) so the row targeting `pending_name_input`'s entry (rename) or the synthetic new-entry row (create) renders the inline editor in place of the label, within the bounded-rows budget (reserve one slot so the editor row always renders)
- [x] 3.2 Render the buffer text with a caret quad and selection-highlight quad (reuse `EditorElement`'s `fill()` caret idiom); register `on_mouse_down(Left)` with `cx.stop_propagation()` on the editor row, mapping click x to a caret byte offset via `estimate_file_tree_text_width`-based char hit-testing (Shift extends selection)
- [x] 3.3 Implement the shared click-away commit: resolve the editor on left mouse-down outside the field — commit through `confirm_pending_name` when the trimmed buffer is non-empty, keep the editor open with the localized refusal status otherwise; add i18n entries for any new status strings (src/i18n.rs)
- [x] 3.4 Replace the under-tab-bar fallback `pending_name_prompt_view` rendering (src/app/root_view.rs "document-workspace-column" branch) with the same editor view + stop-propagation guard, for tab-bar Rename with the sidebar hidden; delete the old panel-top prompt view
- [x] 3.5 In `open_name_prompt` (src/app/workspace.rs), compute the initial selection: for Rename select the base name (final path component up to the last non-leading `.`), for create kinds select the whole prefill; unit-test `report.md` → `report` selected, extension preserved

## 4. Verification

- [x] 4.1 Update existing `PendingNameInput` prefill tests (src/app/tests.rs `pending_name_input_prefill_matches_kind_defaults`) for the new fields and pre-selection policy
- [x] 4.2 Add end-to-end-ish app tests: rename via prompt survives an incidental click elsewhere then commits on Enter; click-away with empty buffer keeps the editor open and shows the refusal status; Escape still cancels without filesystem changes
- [ ] 4.3 Run `cargo test --workspace` and confirm no regressions
- [ ] 4.4 Manual smoke on Windows: F2 rename, right-click Rename, tab-bar Rename with sidebar hidden, CJK filename editing with IME, click-away commit, drag in editor pane while editor open
