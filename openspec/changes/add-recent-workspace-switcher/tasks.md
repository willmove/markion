## 1. Session storage model

- [x] 1.1 Add `MAX_RECENT_WORKSPACES` and a `WorkspaceSnapshot` (`root`, `open_files`, `active_file`) on `SessionState` in `src/model.rs`, plus helpers to touch/cap/dedupe the workspace list and to snapshot only in-root path-backed tabs
- [x] 1.2 Extend `src/storage/session.rs` to read/write `[[workspaces]]`, keep top-level `workspace_root` / `open_files` / `active_file` as aliases of the current snapshot, heal verbatim prefixes on roots and snapshot paths, and synthesize one snapshot from a legacy file that has no workspace list
- [x] 1.3 Add unit tests for round-trip, legacy migration, cap/dedupe, empty-path ignore, and verbatim-path healing of workspace entries

## 2. Persist the current workspace slot

- [x] 2.1 Update `sync_and_persist_session` so the current slot records only in-root Markdown, curated-text, and image tab paths (plus `active_file` when it is in that list); omit untitled/recovery tabs; do not recompute derived Markdown caches
- [x] 2.2 Call that persist path from the existing open/save/close/tab-switch and `set_workspace_root` sites so the current slot stays current without writing on the keystroke path
- [x] 2.3 Add tests that foreign (out-of-root) live tabs are not written into the current snapshot and that text/image paths are included when they sit inside the root

## 3. Explicit workspace switch

- [x] 3.1 Add a workspace-switch entry point that writes the current snapshot, then either no-ops when the target is already current or continues without a dirty prompt
- [x] 3.2 Close only clean tabs through the existing close path; keep every dirty tab (named or untitled) with its text, undo, dirty flag, and recovery snapshot; do not show Save / Don't Save / Cancel on switch
- [x] 3.3 Open the target snapshot through existing open APIs: restore surviving paths in order, reuse an already-open tab for the same path, focus `active_file` when present; use a welcome tab only when no tabs remain; then `set_workspace_root` and async-scan
- [x] 3.4 Prune a selected root that is no longer a directory, report localized failure, and leave the current workspace unchanged
- [x] 3.5 Add tests for restore-remembered, empty-new-folder, dirty tabs kept across switch, vanished root, and current-root no-op

## 4. Open Folder and implicit rebase

- [x] 4.1 Route File → Open Folder (and the header Open Folder action) through explicit workspace switch; reveal the Files sidebar as today; keep picker cancellation non-destructive
- [x] 4.2 Keep implicit rebase (open a file outside the current root) as tree-only: persist the old slot first, change the root, do not replace live tabs with the new root’s stored snapshot
- [x] 4.3 Add tests that implicit rebase leaves the previous workspace’s snapshot intact and does not load the destination snapshot’s tabs

## 5. Launch restore

- [x] 5.1 Change `filter_restorable_session` / `restore_session_on_startup` to restore the current workspace slot (Markdown, text, and image paths) on the background executor, with the same CLI-intent skip for conflicting document/root fields
- [x] 5.2 Keep recovery prompting after session restore and keep `[layout]` restore on CLI launches
- [x] 5.3 Add tests for current-slot restore, missing-path skip, CLI override, and image/text tab restore

## 6. Files header switcher and empty state

- [x] 6.1 Turn the Files-panel workspace-name header into a click switcher (current folder marked, recent roots, Open Folder) without breaking drag-drop onto the workspace root
- [x] 6.2 When no root is established and recent workspaces exist, list those folders in the empty state as the same explicit-switch actions; keep the existing placeholder when the list is empty
- [x] 6.3 Leave File → Open Recent as a file list; do not add a File-menu recent-folders submenu
- [x] 6.4 Add focused UI/wiring tests for header/empty-state actions where practical (string or unit-level)

## 7. Localization

- [x] 7.1 Add `Msg` keys for switcher chrome, empty-state recent-folder copy, and workspace-switch status (success / picker cancel / missing folder) in every supported language
- [x] 7.2 Confirm a missing translation fails at compile time; do not add a workspace-switch unsaved prompt (close-tab and quit keep their existing keys)

## 8. Verification

- [x] 8.1 Run `cargo test --workspace` and fix regressions while preserving per-version Markdown caches, syntax-highlight memoization, cached text handles, bounded file-tree rendering, and GUI-free workspace members
- [x] 8.2 Run `openspec validate add-recent-workspace-switcher` and leave this change apply-ready
