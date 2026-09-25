## Context

See proposal.md for motivation. Observed behavior in the current code:

```
sync ends in conflict
  └─ git_sync.rs finish: git_conflict_admission = Some(ExclusiveAdmission)
        │   (gate.exclusive_operation = Some(id) for the whole repository)
        ▼
active_git_path_locked(path) → git_operations.is_mutating(path) == true
  └─ typing / undo / save / file-tree delete → "Operation running; wait or cancel"
        │
ResolveGitConflict → prepare_conflict_view → ConflictManager::restore_session
  └─ HEAD / MERGE_HEAD differ from checkpoint → ConflictError::StaleSession
        ▼
error branch: app.git_conflict_admission = admission.take()   ← lock restored, never dropped
```

Startup recovery (`arm_git_recovery`) takes an `ExclusiveAdmission` for `ConflictSession`, `OwnedStaging` and some `ExternalState` assessments and keeps it in `git_conflict_admission` or `recovery_guards`. `BackupSyncPresentation` shows `ConflictOrRecovery` whenever `details.recovery` is non-empty, which counts every active journal checkpoint for the repository. `adopt_completed` already retires `CommitCompleted` / `IntegrationCompleted` checkpoints, but nothing calls it automatically, and there is no exit for a checkpoint whose recorded commit has since been superseded by later commits (it assesses as `ExternalState`).

No rendering or document state is touched: the change only affects which writes the admission registry allows and how journal checkpoints are reconciled. Per-version Markdown caches, syntax highlighting and the bounded file tree are unaffected.

## Goals / Non-Goals

**Goals:**
- No code path keeps a repository admission after deciding its session is stale.
- The journal converges to reality without user action when that is safe.
- A waiting conflict blocks only its own paths.

**Non-Goals:**
- Changing how conflicts are detected, merged, committed or pushed.
- Reworking the resolver dialog layout.

## Decisions

**1. Stale-session handling drops the admission and re-assesses.** In `prepare_conflict_view` callers and the finish/abort path in `git_sync.rs`, a `StaleSession` result drops the admission, clears `git_conflict_admission`, `git_ui.conflict` and `conflict_surface_open`, then runs the same reconciliation as startup for that repository. Alternative considered: retry `restore_session` later while holding the lock — rejected, because it recreates exactly the deadlock.

**2. Add a `Superseded` assessment in `crates/git-sync`.** A checkpoint whose `resulting_commit` (or `expected_head` when absent) and `fetched_tip` (when present) are ancestors of HEAD, with no merge in progress and no index conflicts, assesses as `Superseded` even if HEAD moved past it. `IntegrationCompleted` / `CommitCompleted` keep their current meaning. Alternative: treat any non-matching checkpoint as discardable — rejected, the spec requires preserving recovery material when state cannot be matched.

**3. Automatic retirement only without differing drafts.** A reconciliation pass (startup, after each sync completion, after a stale session) retires `CommitCompleted`, `IntegrationCompleted` and `Superseded` checkpoints through `retire_success` when `drafts` is empty or every draft's fingerprint equals the committed blob. Checkpoints with differing drafts become a recovery item with "Open draft" and "Discard record" actions. Discard requires confirmation and calls a new journal `discard(operation_id)` that removes the checkpoint, drafts and its recovery directory.

**4. Split exclusive admission from conflict claims.** `GitOperationRegistry` gains `claim_conflict_paths(identity, operation_id, paths)` returning a guard stored where `git_conflict_admission` is today. `try_write(path)` fails with a new `AdmissionError::ConflictOwned` only for claimed paths; `is_mutating(path)` stays repository-wide but is true only while an `ExclusiveAdmission` exists. A sync that ends in conflict converts its exclusive admission into a conflict claim over `session.files` (plus draft paths) before returning. Finish/abort re-take an exclusive admission for the Git mutation itself. Alternative: keep one exclusive admission and special-case editing — rejected, because every writer (save, autosave, file tree, image operations) would need the special case.

**5. Busy reason is derived, not stored.** `active_git_path_locked` becomes `git_path_lock_reason(path) -> Option<LockReason { Running, Conflict, Settings, Inspection }>`. `Running` keeps the current `GitMsg::Busy` text; `Conflict` uses a new `GitMsg::ConflictNeedsAttention` string with a resolve entry. Settings/inspection keep today's behavior.

## Risks / Trade-offs

- [A path renamed during a conflict escapes the claim] → claims include both `session.files` paths and draft paths; the resolver already refuses rename conflicts in-app.
- [Automatic retirement deletes recovery someone wanted] → only checkpoints whose effect is provably in HEAD and whose drafts equal committed content are retired; everything else stays.
- [Existing journals on user machines] → handled by the startup pass; the user's vault with one `Resolving` and two `Fetching` checkpoints becomes: `Resolving` has a draft → explicit choice; the `Fetching` checkpoint at HEAD → `CommitCompleted` retired; the earlier `Fetching` checkpoint → `Superseded` retired.

## Migration Plan

No data format change beyond the optional `discard`. Rollback is reverting the change; old builds read the retired journal without issue.
