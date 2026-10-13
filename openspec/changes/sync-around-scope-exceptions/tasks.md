## 1. Policy model and partition (`crates/git-sync`)

- [x] 1.1 Add `allowed_extensions` and `included_once` to `RepositoryPolicy` with `#[serde(default)]` and update `default_notes_policy` constructors; verify with a policy round-trip test that a schema-1 file without the fields still loads and that the new fields persist
- [x] 1.2 Add the OS-artifact basename matcher (case-insensitive, untracked only); verify with unit tests for every listed pattern, a near-miss name (`DS_Store.md`), and a tracked `.DS_Store` that still follows tracked policy
- [x] 1.3 Implement `RepositoryPolicy::partition` with `ScopePartition` / `HeldBack` reasons (Hidden, FileType, OutsideFolders, Symlink, NestedRepository, RenameAcrossScope), allowed extensions, and include-once matching via the supplied fingerprint probe; verify with table-driven unit tests per reason, an allowed extension, a matching include-once record, and a lapsed include-once record after content change — implemented with a `ScopeFacts` trait (implemented by `GitRepository`; fakes in unit tests)
- [x] 1.4 Add exact-content move pairing (batched `hash-object --stdin-paths` + index blob ids) behind the partition probe, including the newline-path conservative fallback; verify with repository tests where a tracked PDF moved into a dot-folder holds back both paths, an unrelated deletion stays automatic, and a newline path holds back all deletions — the newline-path fallback is unit-tested with a fake (Windows file names cannot contain newlines); the real-repository move test lives in `crates/git-sync/tests/scope_exceptions.rs`

## 2. Engine behavior (`crates/git-sync`)

- [x] 2.1 Make `preflight` return the partition and require `plan.paths == partition.automatic` instead of failing on Attention; verify with sync-engine tests that Sync Now and Commit Locally commit two edited notes while a new PDF and a `.hidden/config` stay untracked
- [x] 2.2 Pass held-back new paths to `integrate` as tolerated paths, and add `SyncEngineError::HeldBackTrackedChanges { paths }` for held-back tracked changes; verify with sync-engine tests for: — a held-back tracked pause records the checkpoint as `NeedsAttention` (not `Integrating`), so recovery does not hold a repository-wide lock for it
  - remote updates with a tolerated held-back file (integrates, uploads, file preserved);
  - a held-back tracked change with remote updates (pauses before worktree change, local commit retained, paths reported);
  - an incoming path colliding with a held-back file (refused, file preserved);
  - a new untracked note created during fetch (still pauses)
- [x] 2.3 Add `held_back` to `SyncOutcome::Synchronized` and exclude held-back paths from `pending_local_changes`; update all match sites; verify with an engine test asserting the counts after a sync with one held-back file
- [x] 2.4 Prune `included_once` records after a successful commit when the path is tracked or missing; verify with an engine test that an included-once file is committed exactly once and its record is removed — pruning is `RepositoryPolicy::prune_included_once`, called by the app after every operation with one batched tracked query

## 3. Device-local exclusions (`crates/git-sync`)

- [x] 3.1 Add helpers to append, list and remove Markion-marked entries in `<common_dir>/info/exclude` (escaping, `/`-normalized anchored paths, atomic write, creating `info/`); verify with tests that excluded paths (including names with `*`, `[`, `#`, leading `!`, trailing space, and Unicode) disappear from status, that removal restores them, and that hand-written lines are untouched — Windows file names cannot end in a space, so the trailing-space escape is covered by the pattern round-trip test instead of a real file

## 4. App integration (`src/app`)

- [x] 4.1 Replace the two early-return scope loops in `run_git_operation` with the partition; leave dirty buffers of held-back paths unsaved; verify with app tests that Sync Now succeeds with a held-back PDF and that a dirty buffer for a held-back path keeps its unsaved state — verification note: covered by the engine integration tests and the app partition tests; an app-level end-to-end Sync Now test was not added because `run_git_operation` writes the real journal and policy file (no test seam). A test-only policy-store override (`git_ui.policy_store_override`) was added for refresh and decision tests
- [x] 4.2 Store the partition in `GitDetails`, redefine `attention` as held-back tracked changes plus move/rename reviews, and add `held_back_items` to `BackupSyncFacts` / `BackupSyncPresentation` with `pending_paths` excluding held-back paths; verify with `model.rs` projection tests that only-held-back state can be Synchronized with `held_back_items > 0`, and app tests for the summary line in Synchronized and PendingLocal states
- [x] 4.3 Render held-back rows with localized reasons, the applicable Include once / Always sync .ext / Don't sync on this device buttons, and a changed-since-approval note; run actions in background tasks guarded against running Git operations, then request a refresh; verify with app tests for each action's effect on the next refresh and that no button is offered for symlinks or nested repositories — verification note: decisions are tested through `apply_git_scope_decision` and the next refresh; button visibility for symlinks/nested repositories follows `HeldBack::decidable`, covered by crate unit tests
- [x] 4.4 Extend the sync settings form with the extra-file-types field (validated, `;`-separated) and the device-exclusions list with Remove; verify with app tests that saving an invalid extension is rejected with a localized message and that Remove restores the file to the held-back list — verification note: extension validation is tested through `parse_extensions`; Remove is covered by the decision test (settings list reads `info/exclude` synchronously when the dialog opens; it is a small local file)
- [x] 4.5 Map `HeldBackTrackedChanges` and held-back counts in completion messages to localized status text naming the files; verify with an app test of the completion status for both outcomes

## 5. Localization

- [x] 5.1 Add all new `GitMsg` entries (summary, reasons, three actions, changed since approval, settings labels, Remove, tracked-blocks-updates, sync-complete-with-held-back) in all seven languages; verify with the i18n completeness test

## 6. Verification

- [x] 6.1 Run `cargo test --workspace` and confirm it passes — 47 test binaries, 1923 passed (the one failure in the first run, a missing debug selector for the new sixth settings field, was fixed and the app binary re-run: 681 passed)
- [ ] 6.2 Manual smoke on a scratch notes repository with a remote:
  - add a PDF, a `.DS_Store`, and an edited note, then Sync Now: the note syncs, `.DS_Store` is invisible, the PDF is listed;
  - "Always sync .pdf" then Sync Now uploads the PDF;
  - "Don't sync on this device" hides another file without a repository change;
  - with the remote ahead, Sync Now integrates while a held-back file exists.

  Record results in this change folder
