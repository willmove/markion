## Why

A sync that stops in conflict resolution keeps an exclusive repository admission alive for the whole repository. When the repository later moves on (a later sync succeeds, the merge is finished or aborted outside the app, `MERGE_HEAD` disappears), opening the resolver fails with `conflict session no longer owns the repository state`, the error path puts the admission back, and nothing ever releases it. Every note in that repository then refuses typing, undo, save and file-tree delete with "Operation running; wait or cancel before continuing" until the app is restarted, and stale journal entries keep the Backup and Sync status on "Choose which note changes to keep" indefinitely. This was reproduced on a real vault whose journal still holds a five-day-old `Resolving` checkpoint plus two `Fetching` checkpoints whose commits are already in `HEAD`.

## What Changes

- When a conflict session is found to be stale, the app re-assesses that checkpoint against the actual repository, releases the repository admission, and moves the checkpoint to its correct state instead of holding the lock.
- Checkpoints whose recorded work is already contained in `HEAD` and that own no unresolved draft are retired automatically at startup and after each sync; checkpoints that still carry a draft stay recoverable and are offered as an explicit "keep draft / discard record" choice.
- While a genuine conflict is awaiting the user, ordinary edits, saves and file-tree writes are refused only for the conflict-owned paths; other notes in the same repository stay editable. Exclusive whole-repository admission is held only while Git is actually mutating the worktree.
- The busy message distinguishes "Git is working right now" from "an unfinished sync conflict needs attention" and gives a direct entry to resolve or discard it.

Non-goals: redesigning the resolver UI, changing merge/push semantics, or adding automatic resolution of real conflicts. The file-tree scan and Windows task-dialog code are not touched.

## Capabilities

### New Capabilities

None. `git-conflict-recovery` is introduced by the unarchived `add-git-workspace-sync` change; this change adds requirements to that capability path and should be archived after it.

### Modified Capabilities

- `git-conflict-recovery`: stale sessions must release repository admission and reconcile their checkpoint; completed checkpoints are retired; conflict guarding is scoped to conflict-owned paths; busy state names its cause.

## Impact

- `crates/git-sync`: checkpoint assessment gains a "superseded" outcome and a draft-preserving retirement path; admission registry gains path-scoped conflict claims. No `gpui` dependency is added.
- `src/app/git_sync.rs`, `src/app/git_conflicts.rs`, `src/app/git_panel.rs`: stale-session handling, startup/after-sync reconciliation, recovery surface actions.
- `src/app/application.rs` (`active_git_path_locked`), `src/app/documents.rs`, `src/app/workspace.rs`: lock checks consult path-scoped claims.
- `src/i18n/git.rs`: new status strings for the conflict-needs-attention state, in all supported languages.
- Existing journals on users' machines are migrated in place by the startup reconciliation; no manual cleanup is required.
- Cached-per-version Markdown state, syntax highlighting and bounded file-tree rendering are not affected.
