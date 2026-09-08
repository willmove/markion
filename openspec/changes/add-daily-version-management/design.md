## Context

`add-git-workspace-sync` provides the GPUI-free Git process/repository layer, repository operation admission, exact-path staging, local snapshot commits, bounded history/diff reads, and the Sync sidebar. Its everyday UI currently favors policy-driven automatic snapshots: Commit Locally uses generated policy scope/message, while history inspection is repository-wide and historical content can only be saved as a copy.

Daily version management needs a deliberate local workflow alongside automatic sync. The user must be able to choose a coherent group of changed paths, describe it, inspect how a file evolved, compare it to the working version, and recover old text without an implicit disk overwrite. This crosses the Git core and GPUI document lifecycle, so it must preserve external staging, writer admission, path safety, bounded background work, and per-document cache/version invariants.

Data flows are:

1. Repository status -> immutable change identities -> sidebar selection state.
2. Selected paths + authored message -> checked local commit plan -> exclusive repository admission -> selected-buffer save -> exact staging/commit -> refreshed status/history.
3. Active repository path + page cursor -> bounded file-history query -> commit selection -> bounded historical diff/blob read.
4. Historical text + captured tab/path/disk identity -> UI-thread document replacement -> ordinary dirty/undo state -> later explicit save or commit.

Git reads and file reads stay on the background executor. Only the final document mutation and GPUI state updates run on the UI thread. A restore increments the affected document version through the existing edit transaction; derived Markdown state is recomputed lazily for that new version while other tabs keep their cached state.

## Goals / Non-Goals

**Goals:**

- Let users create a local commit with their own nonblank message and an explicit subset of current repository changes.
- Preserve path identity and external index ownership from selection through commit.
- Browse bounded repository history and bounded history for the active file, including renamed-file traversal when Git can follow it.
- Compare a historical file version with the current working version without running external diff drivers.
- Restore a historical text version into the editor as a reviewable, undoable, unsaved change.
- Keep all operations localized, asynchronous, bounded, and consistent with existing Git status/recovery state.

**Non-Goals:**

- Amend, rebase, reset, revert commits, force operations, interactive staging/hunks, branch/stash/tag management, or history rewriting.
- Automatic fetch, pull, or push from version-management actions.
- Editing binary history or silently restoring deleted/renamed paths on disk.
- Replacing the policy-driven one-click Sync Now workflow.

## Decisions

### Use a transient commit draft tied to one status snapshot

The app will keep a commit draft containing message text, selected repository-relative paths, and the status/head identity used to create it. Opening the composer selects policy-eligible unstaged/untracked changes by default while showing every Git change; users can toggle only committable paths. Submitting re-reads status and rejects a changed HEAD, path set, conflict, active operation, or pre-existing staged entry. The core then stages only literal selected paths and verifies the resulting index before committing.

This reuses the existing exact-path `SyncPlan` and commit recovery machinery, with an explicit plan builder that accepts the authored message and selection. A separate ad-hoc `git add`/`git commit` UI path was rejected because it would bypass journal ownership and write admission.

The composer does not support partial hunks. One repository path is the smallest selectable unit because the current recovery record and content fingerprint model are path-based.

### Save only selected named buffers before committing

After exclusive admission drains prior writes, selected open text buffers are saved through the checked manual-save path. An unnamed document cannot be selected. A selected dirty buffer with a disk conflict blocks the commit; unselected dirty buffers remain in memory and are excluded. After saves finish, the status snapshot and fingerprints are rebuilt before staging.

Saving every policy-eligible buffer was rejected because an explicit commit selection should not persist unrelated drafts. Committing stale disk bytes while a selected newer buffer exists was rejected because the result would differ from what the user reviewed.

### Add path-filtered history and working-tree comparison in the core

`GitRepository` will expose a bounded `history_page_for_path` query and a bounded `working_diff_against` query. Both pass path arguments literally after `--`, disable external diff/text conversion, cap output, and return typed truncation/binary state. File history uses a single path with Git's follow behavior; rename traversal is best effort and remains scoped to one file.

History selection stores the full commit OID and repository-relative path, never an abbreviated OID or display string. Pagination is capped per request, and the UI discards results whose repository/path request token is no longer current.

### Restore text through the document edit model

Restore is available only when a selected commit contains a bounded UTF-8 text blob for a path that maps safely into the current worktree. The app opens or focuses that path, verifies the captured disk identity and tab path, asks for explicit confirmation when replacing a dirty buffer, and applies the historical source as one normal document replacement transaction. The tab becomes dirty and its undo history can return to the prior source. No file is written and no Git command mutates the index or HEAD.

For deleted paths, binary blobs, oversized blobs, invalid UTF-8, symlink entries, or a path whose disk identity changed during the read, Restore is disabled and Save a Copy remains available. Direct `git checkout`, `git restore`, and filesystem overwrite were rejected because they can bypass unsaved-buffer protection and make recovery harder to understand.

### Extend the existing Sync sidebar with a Versions page

The existing sidebar remains the single Git surface. Its Changes page gains a local-commit composer, and a Versions page switches between repository history and active-file history. Selecting a commit shows metadata, changed paths, historical/working comparison, Restore in Editor for eligible text, and Save a Copy.

The current automatic Commit Locally action remains available for policy snapshots. The authored composer is labeled Create Local Version so its deliberate selection/message behavior is distinct. All new status and action text uses `Msg` localization keys.

### Treat restore and commit as separate state transitions

Restore creates a dirty editor state but never commits automatically. A subsequent Create Local Version or Sync Now captures it under the same normal rules. Successful commits refresh status, tree decorations, recent/outgoing history, and composer selection; failed commits retain the message/selection so the user can correct the issue.

### Give repository operations a dedicated top-level menu

Both menu implementations will place a localized Repository menu between Export and Help. It owns the existing Show Sync Details, Sync Now, Commit Locally, Check Remote, Pull Updates, Push Commits, Resolve Git Conflicts, and Set Up Git Sync actions. File retains document, tab, preferences, and application-lifecycle commands. The actions and shortcut IDs remain unchanged, so moving their presentation does not invalidate configured bindings or create a second execution path.

Repository describes the full command set more accurately than Sync because the menu includes local commits, repository inspection, and conflict recovery as well as remote transfer. The Sync sidebar keeps its existing name because it is the detailed synchronization surface.

## Risks / Trade-offs

- [A path changes after it was selected] -> Re-read HEAD/status and validate per-path fingerprints immediately before staging; retain the draft and request review on mismatch.
- [Existing staged content belongs to another tool] -> Disable authored commit and preserve the index until it is clean; never reset or absorb external staging.
- [File history across complex renames is incomplete] -> Use Git's single-path follow behavior, display paths returned by Git when available, and describe history as bounded rather than exhaustive.
- [Historical content is large or binary] -> Return metadata/truncation state, disable editor restore, and keep guarded Save a Copy.
- [A restore races with external disk modification] -> Capture and revalidate disk identity before applying the document transaction; stop with the current buffer unchanged on mismatch.
- [Sidebar state grows with large repositories] -> Bound changes, history pages, diffs, and blobs; paginate history and never render or query the complete log synchronously.
- [The new change depends on unarchived Git sync work] -> Reuse its code without duplicating the boundary; validate both changes and leave archival ordering explicit.

## Migration Plan

1. Extend the GPUI-free repository API and fixture tests for file history and working comparisons.
2. Add commit-draft state and checked selected-path plan construction while retaining existing automatic Sync Now behavior.
3. Add the Versions UI, localized strings, safe restore transaction, and regression tests.
4. Move Git and synchronization commands from File to a localized Repository menu in both menu implementations.
5. Update user documentation and run focused crate/root tests plus OpenSpec validation.

The change is additive and stores no new persistent schema. Rolling it back removes the UI/API additions; commits already created remain ordinary Git history, and restored-but-unsaved buffers continue to use the existing session/recovery behavior.

## Open Questions

No product decision blocks the first version. Interactive hunk staging, commit amend/revert, and branch/stash/tag workflows remain separate future changes.
