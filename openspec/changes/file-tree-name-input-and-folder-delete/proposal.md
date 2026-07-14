## Why

The Files sidebar right-click context menu (introduced by the `file-tree-context-menu-actions` change) exposes create / rename / delete, but two of those operations are functionally broken today:

1. **Directory delete always fails.** `FileTree::delete` (`src/storage/file_tree.rs:124`) calls `fs::remove_dir`, which only removes *empty* directories. But every directory that appears in the tree is necessarily non-empty - `collect_file_tree_entries` prunes any folder whose subtree contains no Markdown file, so any folder the user can right-click has at least one Markdown descendant. Consequently "Delete" on any folder returns `Err` ("directory not empty") and the folder is not removed. The action is effectively dead for folders.

2. **Create / Rename apply a hard-coded name with no way to type one.** `create_tree_file` creates `untitled.md`, `create_tree_folder` creates `New Folder`, and `rename_tree_entry` renames to `renamed.<ext>` / `Renamed Folder` - none of them collects a name from the user. Rename is therefore unusable for its stated purpose (it overwrites the existing name with a fixed template), and create forces a follow-up rename just to get a sensible name.

## What Changes

### Directory delete

- `FileTree::delete` SHALL remove non-empty directories recursively (`fs::remove_dir_all`).
- When the delete target is a non-empty folder, the editor SHALL request a *second* confirmation that explicitly warns the folder and all of its contents will be removed (the existing single confirm dialog stays for files and empty folders).
- Delete of a file remains single-confirm; delete of an empty folder remains single-confirm.

### Inline name input for create file / create folder / rename

- Invoking Create File, Create Folder, or Rename from the context menu SHALL open an in-app inline name prompt instead of immediately applying a hard-coded default name.
- The prompt reuses the existing **redirected text-input** pattern already used by the search field and the file-tree filter: a focused `Div` showing `Label: <buffer>` that captures IME keystrokes into a backing `String` via `active_input_text_mut()` / `has_text_input_focus()`.
- `Enter` SHALL commit the typed name: Create File calls `FileTree::create_unique_file`, Create Folder calls `FileTree::create_unique_directory`, Rename calls `FileTree::rename_unique` - all with the user-typed name (sanitized through the existing `sanitize_file_name` path inside `unique_child_path`).
- `Escape` SHALL cancel the prompt without touching the filesystem.
- An empty buffer on commit SHALL be rejected with a localized status message (no entry created).
- Rename SHALL pre-fill the buffer with the current entry name so the user edits rather than retypes.
- The prompt SHALL be dismissed (cancel) when the user clicks elsewhere or opens another menu, mirroring the context menu's own dismissal.

## Non-Goals

- No drag-and-drop file moves.
- No inline in-place editing of the tree cell itself (the prompt is a dedicated focused input line, not an editable row).
- No new file types or Markdown collection changes (the tree stays Markdown-only).
- No changes to document parsing, derived Markdown caches, syntax-highlighting memoization, cached text handles, or undo snapshots.
- No native OS text-entry dialog (GPUI's `window.prompt` is button-only; a native free-text dialog is out of scope).

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `workspace`: the create / rename / delete operations move from hard-coded-name + empty-folder-only to inline-name-input + recursive-folder-delete. The context-menu action set and the Markdown-only tree invariants are unchanged.
- `ui-i18n`: new labels, placeholder/status text, and the second delete-confirm dialog text must be routed through `src/i18n.rs`.

## Impact

- Affected code:
  - `src/storage/file_tree.rs` - `FileTree::delete` switches to `remove_dir_all`; new unit tests for file / empty-folder / non-empty-folder delete.
  - `src/main.rs` - new `PendingNameInput` state integrated into `has_text_input_focus` / `active_input_text_mut` / `after_input_changed`; `create_tree_file` / `create_tree_folder` / `rename_tree_entry` rewritten to open the prompt and commit on Enter; `delete_tree_entry` gains a non-empty-folder second confirm; a name-prompt view is rendered (reusing the `search_field_view` styling); new Enter/Escape actions registered.
  - `src/i18n.rs` - new `Msg` variants (prompt labels, placeholder, empty-name warning, recursive-delete confirm title/detail) with EN + Simplified Chinese strings.
- Affected specs: `workspace`, `ui-i18n`.
- APIs/dependencies: no public API changes, no new dependencies.
- Invariants: file-tree ops touch only the filesystem and app-level tree/tab state; they do NOT touch per-document derived Markdown caches, the syntax-highlighting memo, cached text handles, or undo snapshots. The bounded-row rendering of the tree is unaffected (the prompt is an overlay input, not extra rows). The redirected-text-input contract (`has_text_input_focus` guarding IME routing) is extended, not replaced.
