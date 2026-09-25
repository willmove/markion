## 1. Stale session releases the lock

- [x] 1.1 In `src/app/git_sync.rs` and `src/app/git_conflicts.rs`, when `restore_session` / finish / abort return `StaleSession`, drop the admission and clear `git_conflict_admission`, `git_ui.conflict` and `conflict_surface_open` instead of restoring the admission; verify with an app test that registers a repository, installs a conflict admission, simulates `StaleSession`, and asserts `active_git_path_locked()` is false and a save succeeds afterwards
- [x] 1.2 Trigger a reconciliation pass for that repository right after a stale session (stub that re-runs recovery assessment until group 2 lands); verify the status no longer shows the stale-session error on the next refresh in the same test

## 2. Checkpoint reconciliation in `crates/git-sync`

- [x] 2.1 Add `RecoveryAssessment::Superseded` for checkpoints whose recorded commit and fetched tip are ancestors of HEAD with no merge in progress and no index conflicts; verify with `crates/git-sync` tests covering: commit at HEAD (still `CommitCompleted`), commit behind HEAD (`Superseded`), commit not reachable (`ExternalState`)
- [x] 2.2 Add `JournalStore::discard(operation_id)` that removes the checkpoint, its drafts and its recovery directory; verify with a journal test that the entry and recovery files are gone and other checkpoints are untouched
- [x] 2.3 Add a `reconcile(identity)` helper that retires `CommitCompleted`, `IntegrationCompleted` and `Superseded` checkpoints via `retire_success` when they have no draft or every draft matches the committed blob, and returns checkpoints that need an explicit choice; verify with `cargo test -p markion-git-sync` using a fixture journal shaped like the reported vault (one `Resolving` with draft, one `Fetching` at HEAD, one `Fetching` behind HEAD) and asserting exactly the two `Fetching` entries are retired

## 3. Wire reconciliation into the app

- [x] 3.1 Run `reconcile` in `arm_git_recovery` before taking any admission, and after every sync completion in the finish handler; verify with an app test that a journal with only completed checkpoints yields zero recovery items and no held admission
- [x] 3.2 Add "Open draft" and "Discard record" actions for stale checkpoints with drafts in the recovery surface; "Open draft" opens the draft text as an untitled document and retires the checkpoint, "Discard record" asks for confirmation then calls `discard`; add the new strings to `src/i18n.rs` / `src/i18n/git.rs` for every language and verify with the i18n completeness test plus an app test for each action

## 4. Path-scoped conflict claims

- [x] 4.1 Add `claim_conflict_paths` and `AdmissionError::ConflictOwned` to `GitOperationRegistry`; `try_write` fails only for claimed paths and `is_mutating` is true only while an exclusive admission exists; verify with `admission.rs` tests for claimed path, unclaimed sibling path, and claim release on drop
- [x] 4.2 When a sync ends in conflict, convert the exclusive admission into a conflict claim over the session's files and draft paths; finish/abort re-take an exclusive admission for the Git mutation; verify with an app test that editing and saving an unrelated note in the same repository succeeds while the conflicted note is refused
- [x] 4.3 Make file-tree create/rename/move/delete, save, autosave and image operations report `ConflictOwned` distinctly from `Deferred`; verify with an app test that deleting an unrelated file during a conflict succeeds and deleting the conflicted file is refused with the conflict message

## 5. Busy reason in the status

- [x] 5.1 Replace `active_git_path_locked` with a lock-reason query and use it at every current call site (`editor_element.rs`, `application.rs`, `editing.rs`, `documents.rs`, `workspace.rs`); show `GitMsg::Busy` only for running operations and a new conflict-needs-attention message with a resolve entry otherwise; verify with app tests for both messages and the i18n completeness test

## 6. Verification

- [x] 6.1 Run `cargo test --workspace` and confirm it passes
- [ ] 6.2 Manually reproduce on a copy of the reported vault and journal: start the app, confirm notes are editable and savable immediately, the two completed checkpoints are gone, and the `Resolving` checkpoint is offered as keep-draft / discard; record the result in this change folder
