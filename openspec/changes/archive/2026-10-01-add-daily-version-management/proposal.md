## Why

The synchronization workflow can create automatic snapshots and show bounded history, but it does not yet support the everyday Git loop inside the editor: deliberately grouping changes, writing a meaningful local commit, following one file through history, comparing versions, and restoring an earlier version for review. Adding that loop makes Git useful between remote syncs while keeping recovery and repository safety visible.

## What Changes

- Add an explicit local-commit composer with a required message and selectable repository changes.
- Preserve pre-existing staging arrangements and validate selected paths/content before committing.
- Add repository and current-file history browsing with commit metadata, changed paths, and bounded diffs.
- Add comparisons between the working version and a selected historical version.
- Add safe text restoration that applies historical content to an editor buffer as an unsaved, undoable edit; binary or oversized historical content remains available through Save a Copy.
- Add a localized Repository top-level menu and move every existing Git and synchronization command out of File into that menu in both native and in-window menu bars.
- Refresh Git status, history, file-tree decorations, and open-document state after successful actions.
- Localize the new controls, statuses, errors, and confirmation text and document the workflow.
- Non-goals: amend, rebase, reset, force operations, branch management, stash management, tag management, arbitrary commit-range rewriting, or automatic remote access.

## Capabilities

### New Capabilities

- `git-version-management`: User-authored local commits, repository/file history inspection, version comparison, and safe restoration of historical text.

### Modified Capabilities

None.

## Impact

- Extends the GPUI Git sidebar, native and in-window menu structure, and application command wiring under `src/app/`, with all new user-facing strings routed through localization.
- Extends the GPUI-free `markion-git-sync` repository API for path-limited commit plans, file history, historical comparison, and checked blob access.
- Reuses the repository operation registry, write admission, document lifecycle, bounded Git process output, and recovery mechanisms introduced by `add-git-workspace-sync`.
- Historical inspection remains asynchronous and bounded. Restoring text updates only the affected document version so derived Markdown caches, syntax highlighting, and cached text handles retain their existing per-version behavior.
