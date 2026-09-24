## 1. Duplicate naming and copy

- [x] 1.1 Add a GPUI-free duplicate helper in `src/storage/file_tree.rs` that copies a file or folder beside itself under `{stem} - {marker}{ext}`, then `{stem} - {marker} ({n}){ext}` from `n = 2`, copies symlink nodes without following them, and deletes a destination this operation created when the copy fails. Verify with unit tests for a file, a nested folder including a non-listed child, a name collision, and a failed copy that leaves no partial destination.
- [x] 1.2 Add `FileTreeContextAction::Duplicate` after Rename on the file and folder menus only, with localized menu and status strings in every `src/i18n.rs` language arm, and dispatch it from `handle_file_tree_context_action` on a background executor. On success select the new path and refresh the tree; do not retarget open tabs or read unsaved buffers. Verify the existing context-action scope test lists Duplicate only for file and folder targets, and that `cargo test -p markion` i18n coverage still passes.

## 2. Silent coalesced tree refresh

- [x] 2.1 Add a scan generation to `schedule_file_tree_scan` so a result applies only when it is still the latest scan for the current workspace root, and so an in-flight scan is followed by at most one queued rescan. Verify with a unit or focused test that an older generation cannot replace a newer tree and that a root change drops the stale result. Confirm the scan still runs on the background executor and does not touch document text, dirty state, undo history, or derived Markdown caches.
- [x] 2.2 Add `notify` to the root app package only. While a workspace root is open, recursively watch it from a background task, debounce about 400 ms, and call the silent refresh path. If the watch cannot start, poll about every 2 seconds with the same silent refresh until the root changes. Stop or restart the task when the root changes or clears. Manual Refresh keeps `StatusFileTreeRefreshed`. Verify `cargo test -p markion` covers the silent-versus-manual status split and that `Cargo.toml` does not add `notify` or `gpui` to a `crates/*` member.

## 3. Integration check

- [x] 3.1 Run `cargo test -p markion` and `openspec validate file-tree-duplicate-and-watch`, and confirm both succeed.
