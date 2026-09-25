# Verification

## Automated

- `cargo test --workspace` passes (2026-09-25).
- `crates/git-sync`:
  - `tests/reconcile.rs`: assessment distinguishes `CommitCompleted` / `Superseded` / `ExternalState`; reconcile retires the two `Fetching` records of a vault-shaped journal and keeps the `Resolving` record whose draft differs; `discard_stale` removes a superseded record and refuses unknown ids.
  - `admission.rs`: a conflict claim blocks its path and containing folder, leaves siblings writable, blocks unregister, and releases on drop.
  - `journal.rs`: `discard` removes the checkpoint, draft and recovery files for that operation only.
- App (`src/app/tests.rs`), all against an isolated journal:
  - stale session releases the claim and the conflicted note becomes editable and savable;
  - running sync reports Busy, a held recovery guard reports the conflict-needs-attention message;
  - file-tree delete succeeds for an unrelated file and is refused with the conflict message for the conflicted file;
  - a sync ending in `Resolving` leaves a path claim, not a repository-wide lock;
  - startup recovery retires completed checkpoints and holds no admission;
  - "Open draft" opens a dirty untitled document and clears the record; "Discard record" keeps the record on Cancel and removes it on confirm.

## Reported vault (scripted)

A local clone of the reported vault (HEAD `6e88852`) with a copy of the reported journal, identity paths rewritten to the clone:

| Record | Phase | Assessment | Reconcile |
| --- | --- | --- | --- |
| `sync-23720-…` | Resolving, 1 draft | IntegrationCompleted | kept (draft differs) |
| `sync-22684-…335…` | Fetching | Superseded | retired |
| `sync-22684-…413…` | Fetching | CommitCompleted | retired |

`discard_stale` on the remaining record then leaves no active records. None of these assessments leads startup recovery to take an admission, so the vault is not locked after launch.

## Not yet verified

- Launching the GUI against the real vault and checking editing, saving and the recovery surface by hand.
