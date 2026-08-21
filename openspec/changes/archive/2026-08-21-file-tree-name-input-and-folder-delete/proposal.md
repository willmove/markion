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

- Invoking Create File, Create Folder, or Rename from the context menu SHALL open an in-app inline name editor instead of immediately applying a hard-coded default name.
- The editor reuses the existing **redirected text-input** pattern already used by the search field and the file-tree filter: keystrokes are captured into a backing `String` via `has_text_input_focus()` / the redirected insert path, so IME composition is handled identically.
- The editor renders in place inside the Files panel - replacing the renamed row, or directly below the parent folder row for create actions. When the target row is not visible (filtered out, inside a collapsed folder, or create against the hidden root) the editor falls back to the top of the panel so it is always visible. When the Files panel itself is hidden (tab-bar Rename), it renders under the tab bar.
- The editor is caret- and selection-aware: Rename pre-fills the buffer with the current entry name and pre-selects the base name (the extension is preserved); create actions pre-fill the default name (`untitled.md` / `New Folder`) with the whole name selected, so one stroke replaces it.
- While the editor is open, typing replaces the selection at the caret, and Left / Right / Home / End / Shift+Arrow / Select All / Backspace / Delete edit the name buffer only - the document caret and selection SHALL NOT move (this also fixes the bug where a click in the editor pane while renaming moved the document caret and started a drag selection).
- Clicking inside the field positions the caret (Shift+click extends the selection).
- `Enter` SHALL commit the typed name: Create File calls `FileTree::create_unique_file`, Create Folder calls `FileTree::create_unique_directory`, Rename calls `FileTree::rename_unique` - all with the user-typed name (sanitized through the existing `sanitize_file_name` path inside `unique_child_path`).
- A left click elsewhere (below the menu bar) SHALL commit through the same pipeline as Enter (Explorer semantics), and that same click SHALL NOT also open a file or switch tabs. Opening a menu (context menu or menu bar) cancels the editor without touching the filesystem.
- `Escape` SHALL cancel the editor without touching the filesystem.
- An empty buffer on commit (or click-away) SHALL be rejected with a localized status message and keep the editor open for retry (no entry created).

## Non-Goals

- No drag-and-drop file moves.
- No multi-line name editing (the buffer is a single line; Enter always commits).
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
  - `src/app/` (formerly `src/main.rs` app state) - `PendingNameInput` gains `cursor`/`anchor` caret and selection state integrated into the redirected-input path (`insert_redirected_text`, `pop_text_input`, `delete_text_input_forward`); create/rename handlers open the editor and commit on Enter or click-away; `delete_tree_entry` gains a non-empty-folder second confirm; `file_tree_rows` interleaves the editor row in the tree with a panel-top fallback, and the under-tab-bar prompt covers the hidden-panel case; Left/Right/Home/End/Select* actions route into the editor while it is open.
  - `src/i18n.rs` - new `Msg` variants (prompt labels, placeholder, empty-name warning, recursive-delete confirm title/detail) with EN + Simplified Chinese strings.
- Affected specs: `workspace`, `ui-i18n`.
- APIs/dependencies: no public API changes, no new dependencies.
- Invariants: file-tree ops touch only the filesystem and app-level tree/tab state; they do NOT touch per-document derived Markdown caches, the syntax-highlighting memo, cached text handles, or undo snapshots. Bounded tree-row rendering is unaffected (the editor replaces or inserts at most one row). The redirected-text-input contract (`has_text_input_focus` guarding IME routing) is extended, not replaced.
