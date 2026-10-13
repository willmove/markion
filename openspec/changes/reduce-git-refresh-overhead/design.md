## Context

See proposal.md for motivation. Current cost structure (measured on the reference Windows machine at ~230 ms per `git` spawn):

```
bootstrap ──► arm_git_details_poll ── every 5 s, unconditionally ──► refresh_git_details
                                                                       │ (UI thread) clone text of EVERY open tab
                                                                       ▼ (background)
  GitRepository::discover ............................ 3 processes
  repository.status()
     ├─ git status ................................... 1
     └─ capabilities() ............................... 7 + full worktree walk (contains_nested_repository)
  observed_remote_tip / classify_history ............. 1–5
  history_page (Recent / Outgoing) ................... 1 (+7 resolve_sync_target for Outgoing)
  content_fingerprint(every changed path) ............ SHA-256 of every changed file, every time
  inspect_note_attachments(changed notes) ............ 2 processes per image reference
  RecoveryManager::assess() .......................... status() AGAIN (8 + walk) + per-checkpoint probes
```

A Sync Now repeats `status()` (each 8 processes + walk) about 5–6 times, resolves the sync target (7 processes) 3 times, and runs `check-ignore` once per untracked path. During integration it runs `ls-tree` twice per incoming path.

Constraints:
- Every safety precondition in the `git-sync` / `git-workspace` specs must still be evaluated against fresh state for write operations.
- `markion-git-sync` must stay gpui-free.
- All Git/filesystem work must stay off the UI thread.

Verified Git behaviors this design relies on (Git 2.51):
- `git status --porcelain=v2 -z --untracked-files=all` reports a nested repository, including an empty one, as a single untracked entry with a trailing slash (`? nested/`). Ignored files are never listed.
- `git check-ignore -z --stdin` classifies a NUL-separated batch in one process. It exits 0 when any path is ignored, 1 when none is.
- `git rev-parse --show-toplevel --absolute-git-dir --git-common-dir --is-bare-repository --is-shallow-repository` answers all five in one process.

## Goals / Non-Goals

**Goals:**
- A routine refresh with unchanged repository control files costs a small constant number of Git processes (target ≤ 6) and no worktree walk.
- A no-op Sync Now drops from ~80 to roughly 30 Git processes with identical safety checks.
- No Git process runs on a timer while the window is inactive (besides an enabled background remote check).

**Non-Goals:**
- Skipping the no-op push.
- Changing network timeouts.
- Caching status across write operations.
- Replacing the system Git adapter.

## Decisions

### 1. Replace the 5-second poll with a coalescing, trigger-driven scheduler

`git_ui` gains a small scheduler (`in_flight`, `queued`) behind `request_git_refresh(cx)`. A request while a refresh is in flight sets `queued`; completion starts exactly one follow-up. This replaces the boolean `refreshing` guard, which silently dropped requests.

Triggers:
- **Watcher signal.** The existing `notify` watcher in `arm_file_tree_watch` gets a second, Git-specific relevance filter:
  - any create/remove/rename/data-modify event under the workspace;
  - plus `.git/HEAD`, `.git/index`, `.git/MERGE_HEAD`, `.git/refs/**` and `.git/packed-refs`;
  - excluding `.git/objects/**`, `.git/logs/**` and `*.lock`.

  It feeds a separate channel drained on the file-tree watcher's existing 400 ms debounce tick; the coalescing scheduler absorbs bursts. The file-tree filter keeps today's semantics (it ignores data modifications on purpose).
- **Window activation**, via GPUI `observe_window_activation`. The handler also records `window_active`.
- **Explicit events:** save completion, Git operation completion, workspace switch, and opening Backup and Sync (most of these already call `refresh_git_details`; they now call `request_git_refresh`).
- **Fallback timer:** every 60 s, firing only while `window_active`. If the watcher failed to start (file-tree polling fallback), the interval is 15 s while active.

Why trigger-driven over a longer fixed interval: a longer poll still burns work forever and makes external changes appear stale. Triggers give fresh state when something actually changes. Markion's own refresh is read-only: `GIT_OPTIONAL_LOCKS=0`, no index write, no ref update, so it does not feed back into the watcher.

### 2. Reuse one `GitRepository` per workspace between refreshes

The app keeps the last discovered `GitRepository` together with a **control fingerprint**: size and mtime of `git_dir/HEAD`, `git_dir/index`, `git_dir/config`, `common_dir/config`, `git_dir/shallow`, `git_dir/info/sparse-checkout` and `<worktree>/.gitmodules`, with a missing-file marker. A refresh rediscovers only when the workspace or the fingerprint changed.

Write operations (`run_git_operation`, onboarding, conflict finish) keep discovering a fresh instance, so their capability probe is always fresh. Alternative considered: a process-wide capability cache in the crate keyed by identity. Rejected because ownership and invalidation are clearer with an app-owned handle, and the crate stays free of global state.

### 3. Single-process discovery and a memoized capability probe

- `discover` uses the combined `rev-parse` (one process instead of three) and also records `bare`/`shallow`.
- `GitRepository` holds `Arc<Mutex<Option<CapabilityProbe>>>` so clones share it, and a failed probe is retried instead of being cached. The probe runs once per instance:
  - `git config -z --get-regexp '^(core\.sparsecheckout|extensions\.partialclone|remote\..*\.promisor)$'`, parsed with Git boolean semantics (`true|yes|on|1`), with exit 1 meaning "none";
  - `git ls-files -z --stage`, filtered for mode `160000` gitlinks (bounded output; truncation is treated as unsupported rather than guessed);
  - the existing `git grep -I -l filter=lfs -- *.gitattributes`.

  Filesystem checks remain for `.gitmodules` and `info/sparse-checkout`.
- `status()` no longer calls the probe's commands. It combines the memoized probe with two values derived from the status output itself:
  - `detached_head`: `# branch.head (detached)`;
  - `has_nested_repositories`: any untracked record ending in `/`, or a tracked gitlink from the probe.

  `parse_porcelain_v2` records untracked directory entries before the trailing slash is lost in `PathBuf` conversion.
- `contains_nested_repository` (the full worktree walk) is deleted. This also removes the failure mode where one unreadable subdirectory made every status read fail.
- `capabilities()` remains public and returns `status()?.capabilities`.

**Behavior note:** a nested repository inside an *ignored* directory is no longer reported. Git never transfers it and never offers it as a candidate, so it does not affect synchronization. Every listed unsupported shape (gitlink/submodule, LFS, sparse, partial, shallow, bare, linked worktree, detached HEAD, untracked nested repository) is still detected.

### 4. Slimmer target resolution

`resolve_sync_target` goes from 7 processes to 4:
1. `symbolic-ref`;
2. one `git config -z --get-regexp` for `branch.<escaped>.(remote|merge)`, with regex-escaped branch name;
3. `remote get-url --all`;
4. `remote get-url --push --all`.

The two `get-url` calls are kept because they apply `insteadOf` rewriting. `check-ref-format` runs only when a name contains characters outside a conservative safe set (`[A-Za-z0-9._/-]`, no `..`, `//`, leading `-` or `.`, or trailing `.lock`). Names inside that set are already valid refs. Drift detection is unchanged: the target is still re-resolved at preflight and before push.

### 5. Batch all per-path Git queries

- `GitRepository::ignored_among(&[PathBuf]) -> HashSet<PathBuf>` uses one `check-ignore -z --stdin` (literal paths, NUL-separated).
- `GitRepository::tracked_among(&[PathBuf]) -> HashSet<PathBuf>` uses one `ls-files -z` of the full index, intersected in memory. One deterministic process, independent of the number of queried paths, and immune to pathspec glob interpretation.
- `run_git_operation` uses both batch helpers for dirty buffers instead of per-buffer `path_is_tracked` / `path_is_ignored`.
- `inspect_note_attachments` gains a multi-note entry point that resolves every candidate reference first, then classifies with the two batch helpers. The single-note function becomes a wrapper.
- Engine preflight stops calling `is_ignored` for untracked status records: status never lists ignored paths, so the result was always `false`.
- `incoming_paths` + `validate_incoming_paths` use one `git diff --raw -z --no-renames --no-ext-diff <local> <remote>`:
  - `old mode == 000000` means absent locally;
  - `new mode == 120000` means an incoming symlink (rejected as today);
  - `new mode == 160000` means an incoming gitlink (now rejected too).

  `--no-renames` also lists the old side of a rename, so it gets a recovery preimage. Today `--name-only` with default rename detection drops it.

### 6. Recovery assessment reuses state

`RecoveryManager::assess_with_state(&RepositoryState)` takes the caller's status. When the journal has no active checkpoint for the identity, it returns immediately without any Git process. `assess()` remains as a wrapper for existing callers. The refresh path passes its own state.

### 7. Bounded UI-thread capture and fingerprint caching

- The refresh request captures text only for **dirty** document tabs. Clean documents are read from disk in the background, as already done for attachment inspection (still capped at 2 MB).
- Disk fingerprints are cached in `git_ui` as `path → (len, mtime, sha256)`, shared with the background task via `Arc<Mutex<_>>`. Only files whose size or mtime changed are re-hashed.
- Dirty-buffer fingerprints are cached by `(DocumentInstanceId, version)`, so a dirty document whose current version is already cached is not re-hashed. Its text is still captured, because attachment inspection of changed notes needs the live buffer; the capture is bounded to dirty documents. This mirrors the existing cached-per-version invariant for derived Markdown state.

The commit-draft review flow (`reviewed_fingerprints`) keeps the same values; it simply stops recomputing unchanged ones. Operation-time fingerprints in `run_git_operation` are still computed fresh, because those protect commits.

### 8. Test-only invocation counter

`GitCommandRunner::with_invocation_counter(Arc<AtomicUsize>)` counts spawned processes. Crate and app tests use it to pin the process budgets in the spec without timing-based assertions.

Data flow after the change:

```
watcher(Git filter) ─┐
window activation ───┤
save / op completion ├─► request_git_refresh ─► [in_flight? → queued] ─► refresh task (background)
workspace / panel ───┤                                                     ├─ reuse GitRepository if control fingerprint same
fallback (active) ───┘                                                     ├─ status (1) + memoized probe (0)
                                                                           ├─ observed tip / classify (1–3)
                                                                           ├─ batched ignored/tracked (≤2, only if needed)
                                                                           ├─ assess_with_state (0 without checkpoints)
                                                                           └─ fingerprints: cache hits skip hashing
```

No editor, preview, outline or highlighting state is touched. The only UI-thread change is capturing less text.

## Risks / Trade-offs

- **[A missed watcher event leaves details stale]** → window activation and the active-window fallback poll bound staleness. Write operations never rely on the refreshed snapshot for safety: they re-read status and capabilities themselves.
- **[Workspace is a subdirectory, so `.git` is outside the watched root]** → the Git filter cannot see control files. Activation, save and fallback triggers still apply, and worktree data events inside the workspace still trigger refreshes.
- **[Cached capability probe misses an uncommitted `.gitattributes` LFS edit during refresh]** → the next write operation re-probes before writing, so Sync Now still refuses. The refresh-only status catches up once the index or config changes.
- **[`ls-files -z` output for the tracked set grows with repository size]** → it is bounded (16 MB, as `tracked_paths` today) and is one process. Truncation is reported as an error rather than guessed.
- **[Regex-escaping branch names for `--get-regexp`]** → names outside the safe set fall back to the existing per-key `git config --get` calls.

## Migration Plan

No persisted data changes. Rollback is reverting the change; behavior returns to the 5-second poll.
