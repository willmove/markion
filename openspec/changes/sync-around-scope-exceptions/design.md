## Context

See proposal.md for motivation. Where scope is decided today:

```
run_git_operation (app precheck)
  for change in status.changes:  decide_path(.., ignored=false, symlink=false) != Automatic  → Err("… outside scope")   ← whole sync stops
  for dirty buffer: tracked? tracked_path_allowed : new_path_allowed           → Err("… outside scope")
GitSyncEngine::preflight
  expected = changes where decide_path == Automatic; Attention → Err(StalePlan)                ← whole sync stops
  plan.paths must equal expected
GitSyncEngine::integrate
  requires status.worktree.changes.is_empty()                                                  ← any leftover file blocks merge
git_panel refresh
  attention = changes where decide_path == Attention → red label, counted in attention_items → NeedsAttention state
```

`RepositoryPolicy` has `tracked_roots`, `new_file_rules` (root + classes ⊆ {Markdown, Text, Image}) and `acknowledged_resource_omissions` (the attachment-check acknowledgements). The default policy tracks the whole repository and accepts all three classes everywhere, so in practice exceptions are:
- new files of other types (PDF, Office, audio);
- new files under dot-folders or hidden names;
- symlinks;
- nested repositories.

Status never lists ignored files. Git refuses on its own to overwrite untracked files during a merge, and `validate_incoming_paths` already refuses incoming additions whose path exists locally.

## Goals / Non-Goals

**Goals:**
- A single scope partition shared by the precheck, engine preflight, and panel, so they cannot disagree.
- Approved content always moves forward; exceptions are visible and resolvable in place.

**Non-Goals:**
- Changing the local-version composer (`commit_selected` keeps requiring Automatic decisions for selected paths).
- Changing the attachment-reference check.
- Detecting moves of edited content.

## Decisions

### 1. One partition function in the crate

Add `RepositoryPolicy::partition(&self, changes, probe) -> ScopePartition` in `policy.rs`:

```
ScopePartition {
  automatic: Vec<PathBuf>,              // committed by Sync Now / Commit Locally
  held_back: Vec<HeldBack>,             // listed, never auto-committed
  ignored:   Vec<PathBuf>,              // OS artifacts (untracked only)
}
HeldBack { path, kind: New | Tracked, reason: Hidden | FileType{ext} | OutsideFolders | Symlink | NestedRepository | PairedMove{with} | RenameAcrossScope }
```

`probe` supplies the per-path facts the policy cannot compute itself:
- symlink check;
- content fingerprints, requested only for paths with an include-once record;
- blob ids for move pairing.

These facts come from the repository layer, so the policy stays pure and unit-testable. The engine preflight, `run_git_operation`, and the panel refresh all call it. The engine preflight requires `plan.paths == partition.automatic`, as today with `expected`.

Alternative considered: keep `decide_path` and filter the Attention paths at each call site. Rejected, because the three call sites have already drifted (the app passes `ignored=false, symlink=false`; the engine computes them).

### 2. Held-back semantics in the engine

- Preflight no longer errors on Attention. It returns the partition along with the state.
- Held-back **new** paths captured at preflight are passed to `integrate` as `tolerated`. The clean-worktree check becomes:
  - no staging;
  - every remaining change is untracked and in `tolerated`.

  Anything else (a new edit during fetch, a held-back **tracked** change) fails as before. The existing incoming-collision check and Git's own untracked-overwrite refusal still protect tolerated files.
- A held-back tracked change fails integration with a new `SyncEngineError::HeldBackTrackedChanges { paths }`, so the app can name the file. The local snapshot commit is already durable at that point and stays pending upload.
- `SyncOutcome::Synchronized` gains `held_back: usize`; `pending_local_changes` excludes held-back paths.

Why not commit tracked out-of-scope changes automatically: the spec requires attention for them, and the default policy tracks the whole repository, so this case only arises from explicit user narrowing.

### 3. Exact-content move pairing

Pairing runs only when the partition has both held-back new files and automatic tracked deletions:
1. `git hash-object --stdin-paths` (one process) computes the blob ids of the held-back new files. It applies the same clean filters and line-ending conversion `git add` would.
2. These are compared against the index blob ids of the deleted paths, taken from one `ls-files -z --stage`.

A deletion with an equal blob moves to `held_back` with reason `PairedMove`.

A held-back path containing a newline cannot be passed to `--stdin-paths`. In that case all tracked deletions are held back, the conservative choice.

Similarity-based pairing (edited and moved) is out of scope. The new file is still listed, so nothing is lost silently, but the deletion of an edited-and-moved file can propagate; this risk is listed below.

### 4. OS artifact list

A fixed, case-insensitive basename list applied to **untracked** candidates only, producing `ignored`:
- `.DS_Store`
- `._*` (AppleDouble files)
- `.AppleDouble`
- `Icon\r`
- `.Spotlight-V100`, `.Trashes`, `.fseventsd`, `.TemporaryItems`
- `Thumbs.db`, `ehthumbs.db`, `ehthumbs_vista.db`
- `desktop.ini`
- `$RECYCLE.BIN`
- `~$*` (Office owner files)
- `.~lock.*#` (LibreOffice locks)

Tracked copies follow the tracked policy, per the spec's ignore-rule semantics. Alternative considered: writing these to `.gitignore`. Rejected, because it would create a repository change that itself needs a decision.

### 5. Three per-file decisions

All three are offered on held-back **new** files that are not symlinks or nested repositories.

- **Include once**: appends `ScopeException { path, content_fingerprint }` (SHA-256, the existing struct) to the new `RepositoryPolicy.included_once` field. The partition treats the path as automatic only while its current fingerprint matches. Records are pruned after a successful commit, once the path is tracked or missing. Stale records (fingerprint mismatch) stay listed as "changed since approval" until the user decides again.
- **Always sync .ext**: adds a normalized extension (lowercase ASCII alphanumeric, 1–16 chars) to the new `RepositoryPolicy.allowed_extensions: BTreeSet<String>`. Applied within `new_file_rules` roots in addition to the built-in classes. It does not override the hidden-path, symlink, or nested-repository rules. Offered only when the reason is `FileType`.
- **Don't sync on this device**: appends a block to `<common_dir>/info/exclude` and writes it atomically (`persist::atomic_write`), creating `info/` if absent. The block is:

  ```
  # markion: device-local exclusion
  /<escaped path>
  ```

  Escaping: `\` before `*?[\`; a leading `#`/`!` escaped; trailing spaces escaped; separators normalized to `/`.

  `info/exclude` is used rather than `.gitignore` because it is local-only, is never committed, and needs no further scope decision. Settings list entries that follow the Markion marker comment and can remove them (marker plus entry). Entries the user wrote by hand are left untouched.

Both new policy fields use `#[serde(default)]`, and `POLICY_SCHEMA_VERSION` stays 1.

### 6. App and presentation

- `run_git_operation` replaces both early-return loops with the partition. Dirty buffers whose paths are held back are not saved, honoring "excluded buffers remain untouched". The plan uses `partition.automatic`.
- `GitDetails` stores the partition. `attention` becomes "held-back tracked changes plus rename/move reviews". The panel's changes list renders held-back rows with:
  - a localized reason;
  - the applicable decision buttons;
  - a "changed since approval" note for lapsed include-once records.

  Actions run in a background task under the same "no Git operation running" guard as settings saves, then request a refresh.
- `BackupSyncFacts` gains `held_back_items`. `pending_paths` excludes held-back paths, and `attention_items` excludes held-back new files. `BackupSyncPresentation` exposes `held_back_items`, and the status surface renders "N files are not included in sync · Review" whenever it is non-zero, under any primary state.
- The settings form gains two lists:
  - extra file types: editable, `;`-separated like the root fields;
  - device-local exclusions: read-only list with a Remove button per entry.

### 7. Strings

New `GitMsg` entries in all seven languages:
- held-back summary;
- one reason per `HeldBack` reason;
- Include once / Always sync .{ext} files / Don't sync on this device;
- changed since approval;
- extra file types label, device exclusions label, Remove;
- held-back-tracked-blocks-updates message;
- sync complete with N files not included.

## Risks / Trade-offs

- **[An edited-and-moved file propagates its deletion]** → the new path is listed as not included, and Version History retains the old content. Similarity pairing is a possible follow-up.
- **["Always sync .ext" admits arbitrary binaries]** → it is an explicit per-extension user decision, shown in settings and removable. Symlinks, nested repositories, and hidden paths are never admitted by it.
- **[Device-local exclusions are invisible to other devices]** → intended. The action label says "on this device".
- **[Older builds rewrite the policy file and drop the new fields]** → approvals are re-requested. No data is lost, because held-back files stay on disk.
- **[A user relies on a held-back tracked change being committed]** → integration pauses with the file named. Committing it requires widening tracked roots or the local-version composer, as today.

## Migration Plan

No migration. Existing policies deserialize with empty `allowed_extensions` and `included_once`. Rollback restores today's blocking behavior; device-local exclusions written to `info/exclude` remain valid Git ignore entries.
