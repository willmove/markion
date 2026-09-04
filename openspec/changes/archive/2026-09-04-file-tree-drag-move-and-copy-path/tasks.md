## 1. Harden file-tree move API

- [x] 1.1 In `src/storage/file_tree.rs`, tighten `FileTree::move_entry` so the source must exist inside the workspace root (not the root itself), the destination parent must be the root or a directory inside it, a folder cannot move into itself or a descendant, and a same-name child in the destination is an `AlreadyExists` error instead of a unique rename.
- [x] 1.2 Add unit tests for those guards (cycle, outside-root, name collision, same-parent no-op or error) and keep the existing move-into-empty-folder success case.

## 2. Localization

- [x] 2.1 Add `Msg` variants for Copy Path, Copy Relative Path, move success, move failure, name-collision, invalid-move, save-before-move, and relative-path copy success/failure.
- [x] 2.2 Provide translations for every supported language and update the exhaustiveness guard in `src/i18n.rs`.

## 3. Copy Path and Copy Relative Path

- [x] 3.1 Add `CopyPath` and `CopyRelativePath` to `FileTreeContextAction` and include them on file and folder menus (after Show in System File Manager, before Refresh), not on the workspace/blank-space menu.
- [x] 3.2 Wire handlers in `handle_file_tree_context_action`: write the platform-normal absolute path or workspace-relative path to the clipboard, reuse/extend the tab-bar copy-path helper as needed, and report localized status without touching document text or caches.
- [x] 3.3 Update `file_tree_context_actions_are_scoped_by_target_kind` (and add a focused relative-path helper test) so file/folder menus include the new actions and the workspace menu does not.

## 4. File-tree drag and drop

- [x] 4.1 Add `DraggedFileTreeEntry` and a drag-session flag so a started drag suppresses the row's left `mouse_up` open/toggle; disable dragging while the inline name editor is open.
- [x] 4.2 Attach `on_drag` to file and folder rows and `on_drop::<DraggedFileTreeEntry>` to folder rows, file rows (resolved to the file's parent folder), the workspace-root header, and blank Files-panel space. Highlight a valid hover target.
- [x] 4.3 Implement the drop handler: validate target, refuse dirty remapped tabs with `StatusSaveBeforeMove`, call `move_entry`, refresh the tree, remap clean open tabs through the existing rename pipeline, and set localized status. Do not recompute derived Markdown caches except for remapped tabs.

## 5. Verification

- [x] 5.1 Add or extend app-level tests for move remapping of a clean open tab, refusal when a remapped tab is dirty, clipboard status for both copy actions, drop onto a file resolving to that file's parent, and drop onto a sibling file as a no-op.
- [x] 5.2 Run `cargo fmt`.
- [x] 5.3 Run `cargo test --workspace`.
- [x] 5.4 Run `openspec validate file-tree-drag-move-and-copy-path`.
- [x] 5.5 Manually verify: drag file/folder into a folder, onto a file inside a folder, and onto the workspace root; confirm invalid drops do nothing; confirm a drag does not also open/toggle; right-click copy absolute and relative paths.
