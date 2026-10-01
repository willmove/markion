## Why

The file tree can create, rename, and delete entries, but it cannot duplicate one in place, and it only updates when the user refreshes or the editor itself changes the tree. Files added or removed outside Markion stay invisible until a manual refresh.

## What Changes

- Add a **Duplicate** item (`创建副本`) to the file and folder context menus. It copies the right-clicked entry beside itself under a free localized name and refreshes the tree.
- Watch the open workspace folder for filesystem changes and refresh the file tree automatically. If a watcher cannot be started, fall back to a timed refresh of the same tree.
- Keep manual Refresh. Automatic refreshes update the tree silently and do not rewrite the status bar.
- Non-goals: no duplicate action on blank workspace space; no copying of unsaved editor buffers; no reload of open document text, dirty state, undo history, or derived Markdown caches when the tree refreshes; no new file types.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `workspace`: file and folder context menus gain Duplicate, and the file tree stays in sync with on-disk changes under the open workspace root.

## Impact

- Affected code: `src/app/mod.rs` and `src/app/workspace.rs` (context action), `src/storage/file_tree.rs` (duplicate naming and copy), `src/app/application.rs` (watch or poll lifecycle around the existing background scan), `src/i18n.rs` (labels and status text).
- Affected specs: `workspace`.
- Dependencies: a filesystem watcher crate on the root app package (no `gpui` dependency in `crates/*`).
- Invariants: the file tree still scans on a background executor and renders a bounded number of rows per frame. Duplicate and automatic refresh do not recompute per-version Markdown caches, syntax highlighting, or cached text handles.
