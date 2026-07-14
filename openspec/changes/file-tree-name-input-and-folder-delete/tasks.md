## 1. Fix recursive folder delete

- [x] 1.1 In `src/storage/file_tree.rs`, change `FileTree::delete` to use `fs::remove_dir_all` for directories instead of `fs::remove_dir`; keep `fs::remove_file` for files and the `ensure_existing_path_within_root` guard and `self.refresh()` call unchanged.
- [x] 1.2 Add unit tests in `src/storage/file_tree.rs` covering: deleting a file, deleting an empty folder, and deleting a non-empty folder (with a nested Markdown file) - the non-empty case must now succeed and the tree must refresh.
- [x] 1.3 Run `cargo test -p markion` (or `cargo test` for the root package) to confirm the new delete tests pass and no existing file-tree tests regress.

## 2. Second confirmation for non-empty folder delete

- [x] 2.1 In `src/i18n.rs`, add `Msg` variants for the recursive-folder delete dialog: a title (e.g. `DialogDeleteFolderRecursiveTitle`) and a detail (`DialogDeleteFolderRecursiveDetail`) that interpolates the folder path. Add English and Simplified Chinese strings; reuse existing `DialogButtonDelete` / `DialogButtonCancel`.
- [x] 2.2 In `src/main.rs` `delete_tree_entry`, after the first confirm resolves to "delete", if the target path is a non-empty directory (detect via `fs::read_dir` non-empty), show a second `window.prompt` (`PromptLevel::Warning`) using the new recursive-delete title/detail; proceed to `FileTree::delete` only if both confirmations are accepted. Files and empty folders keep the single confirm.
- [x] 2.3 Verify the spawned delete task resets any tab whose document path is inside the removed folder (not just exact-match) so a nested open file is reset to a fresh untitled document.

## 3. Pending name-input state and IME routing

- [x] 3.1 Add a `PendingNameInput` struct (`kind: PendingNameKind { CreateFile, CreateFolder, Rename }`, `parent: PathBuf`, `target: Option<PathBuf>`, `buffer: String`) and a `pending_name_input: Option<PendingNameInput>` field on `MarkionApp`, initialized to `None`.
- [x] 3.2 Integrate the prompt into the redirected-input trio: `has_text_input_focus()` returns true when a name is pending; `active_input_text_mut()` returns `&mut pending_name_input.buffer` when pending; `after_input_changed()` shows a localized "typing name" status hint and does not re-run search/tree filtering.
- [x] 3.3 Clear `pending_name_input` in the existing dismissal paths: `close_menu`, `toggle_menu`, `show_file_tree_context_menu`, and any other place that clears `file_tree_context_menu` / `active_menu`.

## 4. Name-prompt view and actions

- [x] 4.1 Render the inline name prompt as an overlay in the Files panel when `pending_name_input.is_some()`, reusing the `search_field_view` styling (`Label: <buffer>`, blue border when active). It is not a tree row, so bounded-row rendering is unaffected.
- [x] 4.2 Register a `ConfirmPendingName` GPUI action bound to `enter`; on commit, if the buffer is empty emit a localized `StatusNameRequired` warning and keep the prompt open, otherwise branch on `kind` (CreateFile -> `create_unique_file`, CreateFolder -> `create_unique_directory`, Rename -> dirty-guard then `rename_unique`) with the typed name, then clear the prompt, set `selected_tree_path`, and refresh.
- [x] 4.3 Route the existing `escape` binding through a dispatcher that cancels the name prompt if one is open, otherwise falls back to the current `ClearFileTreeSearch` behavior.
- [x] 4.4 Add `Msg` variants and EN/ZH strings for the prompt label, placeholder/default-name status, and `StatusNameRequired` warning.

## 5. Wire context-menu actions to the prompt

- [x] 5.1 Rewrite the `CreateFile` branch of `handle_file_tree_context_action` to open the prompt (set `pending_name_input` with `kind = CreateFile`, the resolved parent, buffer pre-filled with `untitled.md`) instead of calling `create_tree_file` inline.
- [x] 5.2 Rewrite the `CreateFolder` branch to open the prompt with `kind = CreateFolder` and buffer pre-filled with `New Folder`.
- [x] 5.3 Rewrite the `Rename` branch to open the prompt with `kind = Rename`, `target = path`, `parent = path.parent()`, and buffer pre-filled with the entry's current file name.
- [x] 5.4 Keep the existing keyboard shortcuts (`CreateTreeFile`, `CreateTreeFolder`, `RenameTreeEntry` actions) wired so they open the prompt against the selected entry / workspace root.

## 6. Reload open tabs on rename (preserve existing behavior)

- [x] 6.1 In the `ConfirmPendingName` Rename commit path, after `rename_unique` succeeds, reload any tab whose document path equals the old path from the new path in place (matching the current `rename_tree_entry` logic), and leave non-matching tabs untouched.

## 7. Tests and verification

- [x] 7.1 Add a focused test that the name prompt is opened for each kind with the expected pre-filled buffer and parent, and that commit/cancel correctly creates / does-not-create entries (use a temp workspace).
- [x] 7.2 Add a test that deleting a non-empty folder now succeeds at the `FileTree::delete` level (covered by 1.2) and that the second-confirm gating is exercised at the app level where feasible.
- [x] 7.3 Ensure the existing `file_tree_context_actions_are_scoped_by_target_kind` test still passes (action set unchanged).
- [x] 7.4 Run `cargo fmt`, `cargo test`, and `openspec validate file-tree-name-input-and-folder-delete`.
- [ ] 7.5 Manually verify: New File / New Folder / Rename open the prompt, Enter creates/renames with the typed name, Escape cancels, deleting a non-empty folder asks twice and removes recursively, deleting a file/empty folder asks once.
