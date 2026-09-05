## Why

Markion already restores the last workspace root and saved Markdown tabs on launch, but it keeps only one global snapshot and treats the Files-panel workspace name as a static label. Users who move between a few folders still have to re-open that folder and rebuild its tab set by hand. The missing piece is a bounded recent-workspace list, shown on the file-tree header, that restores each folder’s last path-backed tabs when selected.

## What Changes

- Persist a bounded list of recent workspace roots in `session.toml`, each with its own ordered path-backed tab list and active-file path (Markdown, curated text, and supported images). The current workspace remains the launch restore target.
- Make the Files-panel workspace-name header a switcher: current folder, recent folders, and Open Folder. Choosing a remembered folder snapshots the current workspace, closes only **clean** tabs, keeps every dirty tab on the tab bar (no prompt), then opens that folder’s last path-backed list and rescans the tree. Save / Don't Save / Cancel stays on close-tab and quit, not on workspace switch.
- File → Open Folder of a remembered directory follows the same restore path. Open Folder of a new directory snapshots the current workspace, closes clean tabs, keeps dirty tabs, and shows a welcome tab only when no tabs remain.
- Opening a file outside the current workspace still rebases the file-tree root only and does not replace the live tab set. Untitled / welcome tabs stay out of snapshots; unsaved buffer text stays with crash recovery.
- Localize picker labels, empty-state recent-folder hints, and switch status.

Non-goals: cursor / selection / scroll restore; per-workspace window layout; multi-root workspaces; persisting file-tree expand/collapse or the filter query; a File-menu recent-folders submenu (the Files header is the switcher); changing Preferences reset to wipe session data.

Invariants preserved: per-document derived Markdown caches, syntax-highlight memoization, cached editor text handles, bounded file-tree rendering, and GUI-free workspace members. Session IO stays off the keystroke path.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `workspace`: Promote session restore from one global snapshot to per-workspace tab snapshots; add a recent-workspace switcher on the Files header; define explicit-switch vs implicit-rebase behavior and dirty-tab handling.
- `chrome-platform`: Include recent-workspace and per-workspace tab paths in the existing platform-normal path persistence rule.
- `markdown-editing`: Replace the stale “tabs are not persisted across launches” rule with per-workspace path-backed tab restore (still no cursor, undo, or untitled-buffer persistence).
- `image-file-viewing`: Path-backed image tabs participate in per-workspace snapshots and restore.
- `ui-i18n`: Route workspace-switcher labels, recent-folder empty state, and switch status through the i18n layer.

## Impact

- Persistence: extend `src/storage/session.rs` and `SessionState` in `src/model.rs` with a bounded recent-workspace table; keep using the existing atomic `session.toml` write; migrate a legacy single-snapshot file into one workspace entry on load.
- App state / documents: snapshot and restore helpers beside the current `restore_session_on_startup` / `sync_and_persist_session` path; explicit workspace switch closes clean tabs through existing close APIs, keeps dirty tabs, and opens the target snapshot (reusing an already-open dirty tab for the same path).
- File tree / chrome: Files header becomes a localized dropdown; empty-state can list recent workspaces; File → Open Folder distinguishes remembered vs new roots.
- i18n: new `Msg` keys for every supported language.
- Tests: session schema/migration, switch restore, dirty tabs kept across switch, implicit rebase, missing-path pruning, localization completeness.
- No new crate, dependency, or Preferences schema.
