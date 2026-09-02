## 1. Replace-eligibility and non-explicit opens

- [x] 1.1 Change `EditorTab::is_safe_to_replace` so a document tab is replaceable only when `!is_dirty()`, and image tabs remain replaceable (`src/app/state.rs`). Update the `open_in_current_tab` / `default_open_intent` comments that still call untitled tabs always safe (`src/model.rs`, `src/app/workspace.rs`, `src/app/application.rs`).
- [x] 1.2 Stop wrapping File → Open in `confirm_discard_then`; always pick a path and call `open_supported_path` with `default_open_intent()` (`src/app/documents.rs`). Leave File → New on the existing two-button discard helper.
- [x] 1.3 Add tests: preference on × pristine untitled/welcome still `ReplaceActive`; untitled dirty and named dirty both `OpenInNewTab`; a tree/recent/File-Open-intent open against an untitled dirty tab appends and preserves text, undo, recovery, and dirty flag. Do not recompute derived Markdown caches in these paths.

## 2. Localized three-way close copy

- [x] 2.1 Add `Msg` variants for Save, Don't Save, close-tab title/detail, and quit/window-close title/detail (including a dirty-count interpolation where needed) in all seven language blocks plus the completeness list in `src/i18n.rs`. Close-tab copy MUST NOT reuse `DialogDiscardNewDetail`.

## 3. Close tab: Save / Don't Save / Cancel

- [x] 3.1 Add a shared unsaved-choice prompt helper using `PromptButton::ok` (Save), `PromptButton::other` (Don't Save), and `PromptButton::cancel` (Cancel) so Windows button IDs stay unique (`src/app/editing.rs`).
- [x] 3.2 Route `close_tab` through that helper. Save persists the targeted tab via existing `save` or Save As (capture `TabContextTarget`; abort on failure, external conflict, or cancelled picker). Don't Save calls today's `close_tab_confirmed`. Cancel mutates nothing. Image tabs still skip the prompt.
- [x] 3.3 Tests: closing a clean or image tab still has no prompt; dirty close source uses the three-button helper; Save-then-close of a named dirty tab writes the file, clears dirty, and removes the tab; Don't Save still deletes the recovery snapshot; Cancel leaves text and dirty flag; last-tab close still leaves a fresh untitled document.

## 4. Quit and window close

- [x] 4.1 Route `request_quit` and `install_window_close_guard` through the same three-button helper and Save-all walker (named `save` in opening order, untitled tabs activated then Save As). Keep `confirming_close` set for the whole picker chain. Failed save, conflict, or cancelled Save As aborts exit.
- [x] 4.2 Don't Save and a successful Save-all both call `discard_all_tab_recovery_files` (menu Quit must no longer discard only the active snapshot). Cancel clears `confirming_close` and leaves every tab unchanged.
- [x] 4.3 Tests: any-tab dirty detection still gates quit; Don't Save cleanup covers every dirty tab's recovery file; Save-all of two named dirty tabs writes both before teardown would run; a cancelled Save As leaves dirty tabs open. Preserve per-version caches on tabs that remain open.

## 5. Spec-tree alignment

- [x] 5.1 Rewrite the unarchived `open-documents-in-current-tab` markdown-editing and image-file-viewing deltas so replace-eligibility is image-or-non-dirty (not "untitled") and File → Open no longer dirty-guard-then-replaces.
- [x] 5.2 `openspec validate protect-unsaved-edits` passes and `cargo test --workspace` is green.
