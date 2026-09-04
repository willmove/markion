## Why

The Files sidebar already lists the workspace and offers create, rename, and delete, but it still cannot move an entry by dragging it into another folder — the `workspace` spec explicitly leaves that as a future candidate even though `FileTree::move_entry` already exists. Users also cannot copy a tree entry's absolute or workspace-relative path from the file-tree context menu; only the tab-bar menu copies an open file's path. Both gaps force a detour through the OS file manager for everyday reorganization and path-sharing.

## What Changes

- Left-dragging a file or folder row in the file tree and dropping it onto a folder, onto a file inside a folder, or onto the workspace root SHALL move that entry into the destination folder on disk, then refresh the tree.
- Right-clicking a file or folder in the file tree SHALL offer **Copy Path** and **Copy Relative Path**, writing the platform-normal absolute path or the workspace-relative path to the clipboard with localized status feedback.
- Open tabs whose paths were the moved entry (or descendants of a moved folder) SHALL be remapped to the new locations, reusing the existing rename remapping pipeline.
- The `workspace` spec SHALL drop the "UI move is not supported" carve-out now that the move API is being surfaced.

Non-goals: no multi-select drag, no copy-on-drop (Ctrl/Option duplicate), no OS-originated drops onto the tree (those remain editor/preview open-file drops), no Properties dialog, and no new file types or scan-filter changes. The leftover unarchived change `add-file-tree-entry-context-actions` also proposed path copy plus Properties; this change takes only the path-copy slice and leaves Properties out.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `workspace`: file-tree rows become draggable move sources; folder rows, file rows (resolved to the file's parent folder), and the workspace root become drop targets; file and folder context menus gain Copy Path and Copy Relative Path; the existing "moving via the UI is not supported" language is removed.
- `ui-i18n`: new context-menu labels and move/copy status messages MUST route through the i18n layer.

## Impact

- Affected code: `src/storage/file_tree.rs` (harden `move_entry` guards), `src/app/root_view.rs` (row drag/drop wiring), `src/app/workspace.rs` and `src/app/mod.rs` (move handler, context actions, drag value type), `src/i18n.rs` (labels and status), focused tests in `src/storage/file_tree.rs` / `src/app/tests.rs` / `src/lib.rs`.
- Affected specs: `workspace`, `ui-i18n`. Clipboard path form already follows `chrome-platform` (no `\\?\` prefix) and is not restated there.
- APIs/dependencies: no public API or new crate. Reuses GPUI `on_drag` / `on_drop` (already used for Visual Edit block reorder and pane splitters) and `FileTree::move_entry`.
- Invariants: file-tree bounded row rendering stays intact; derived Markdown caches, syntax-highlight memoization, cached text handles, and undo snapshots are untouched except for the existing rename-style tab remapping when a moved path is open.
