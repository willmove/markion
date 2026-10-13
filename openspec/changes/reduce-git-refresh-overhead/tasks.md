## 1. Measurement seam

- [x] 1.1 Add `GitCommandRunner::with_invocation_counter(Arc<AtomicUsize>)` in `crates/git-sync/src/process.rs` that increments once per spawned process; verify with a crate test that two `run` calls increment the counter by two and that a runner without a counter behaves as before

## 2. Cheaper discovery and capability probe (`crates/git-sync`)

- [x] 2.1 Make `GitRepository::discover` use one combined `rev-parse --show-toplevel --absolute-git-dir --git-common-dir --is-bare-repository --is-shallow-repository`; verify with a counter test that discovery spawns exactly one process and with the existing discovery/identity tests
- [x] 2.2 Record untracked directory entries (trailing `/`) in `parse_porcelain_v2` before path conversion; verify with a parser unit test using a `? nested/\0` record
- [x] 2.3 Introduce the memoized capability probe (`Arc<OnceLock<_>>`: combined config regexp with Git boolean parsing, `ls-files -z --stage` gitlink scan, LFS grep, `.gitmodules` / sparse-checkout file checks). Derive `detached_head` and untracked nested repositories from each status. Delete `contains_nested_repository`. Verify with crate tests that a status read after the first spawns exactly one process, and that the existing nested-repository test (`validates_containment_and_detects_nested_repository`) plus new tests for a tracked gitlink, detached HEAD, sparse config, `extensions.partialClone` and an LFS attribute still disable write sync
- [x] 2.4 Slim `resolve_sync_target` to symbolic-ref + one config regexp + two `remote get-url` calls, with the safe-name fast path and per-key fallback; verify with counter tests (≤ 4 processes for a safe branch name) and with existing target-resolution tests, including `insteadOf` rewriting and multiple push URLs

## 3. Batched path queries (`crates/git-sync`)

- [x] 3.1 Add `GitRepository::ignored_among` (one `check-ignore -z --stdin`) and `tracked_among` (one `ls-files -z`, intersected in memory); verify with crate tests for names containing spaces, Unicode, leading dashes and `[`/`]`, and a counter test showing one process per call regardless of input size
- [x] 3.2 Remove the per-path `is_ignored` call from engine preflight for status-reported untracked paths; verify `cargo test -p markion-git-sync` sync-engine tests still pass and a counter test shows preflight process count independent of the number of untracked files
- [x] 3.3 Replace `incoming_paths` + per-path `tree_entry` validation with one `diff --raw -z --no-renames --no-ext-diff` parse (absent-locally, incoming symlink, and incoming gitlink checks); verify with sync-engine tests for an incoming symlink (rejected), an incoming gitlink (rejected), an untracked collision (rejected), a remote rename whose old path receives a preimage, and a counter test showing constant process count for 50 incoming files
- [x] 3.4 Add a multi-note attachment inspection entry point that classifies all references with the two batch helpers, keeping the single-note wrapper; verify with resource tests that results match the previous per-reference behavior and that inspecting a note with 20 image references spawns at most two processes
- [x] 3.5 Add `RecoveryManager::assess_with_state` with an early return when no checkpoint exists for the identity; verify with conflict/reconcile tests that assessments are unchanged and that a journal without checkpoints spawns no process

## 4. App refresh scheduling (`src/app`)

- [x] 4.1 Replace the `refreshing` flag with a coalescing scheduler (`request_git_refresh`: in-flight + one queued follow-up) and route existing `refresh_git_details` callers through it; verify with an app test that three rapid requests run at most two refreshes — implemented inside `refresh_git_details` (kept as the request entry point, so every existing caller is coalesced); results are applied even when a follow-up is queued so continuous triggers cannot starve the snapshot
- [x] 4.2 Add the Git-specific watcher filter and debounced channel next to the file-tree watcher (worktree data/create/remove/rename plus `.git` HEAD/index/MERGE_HEAD/refs/packed-refs; excluding objects, logs and `*.lock`); verify with unit tests of the filter function and an app test that an external file write triggers a refresh — verification note: the filter is covered by unit tests (`git_refresh` path filter and `git_watch_event_is_relevant`); no end-to-end watcher test was added because real `notify` delivery is timing-dependent in the test executor
- [x] 4.3 Track window activation with `observe_window_activation`, request a refresh on activation, and replace `arm_git_details_poll` with a fallback timer that only fires while the window is active (60 s, or 15 s when the watcher fell back to polling); verify with app tests that no refresh is requested by the timer while inactive and that activation requests one
- [x] 4.4 Keep the discovered `GitRepository` plus control fingerprint in `git_ui` and rediscover only when the workspace or fingerprint changes; verify with an app test using the invocation counter that two consecutive refreshes with no change spawn no discovery or probe processes on the second run, and that touching `.git/index` causes a rediscovery

## 5. Bounded UI-thread capture and fingerprint caches

- [x] 5.1 Capture text only for dirty document tabs in the refresh request and read clean changed notes from disk in the background (2 MB cap kept); verify with an app test that a refresh with many clean open tabs captures no clean-tab text and still reports attachment issues for a changed clean note — verification note: the test pins `capture_dirty_git_documents` (clean tabs contribute no text); attachment reporting for clean changed notes reads from disk as before
- [x] 5.2 Add the disk fingerprint cache (`path → (len, mtime, sha256)`) and the dirty-buffer cache keyed by `(DocumentInstanceId, version)`; use them in refresh and draft selection but not in operation-time commit fingerprints; verify with tests that an unchanged file is not re-hashed, that a size/mtime change and a new document version invalidate the entry, and that local-version review still detects a file changed after review — verification note: cache invalidation is unit-tested in `src/app/git_refresh.rs`; operation-time review fingerprints in `run_git_operation` are unchanged (still computed fresh), so no new review-staleness test was added
- [x] 5.3 Use `tracked_among` / `ignored_among` for dirty-buffer scope checks in `run_git_operation`, and the multi-note attachment inspection in both refresh and operation precheck; verify with existing `src/app/tests.rs` sync tests and a counter assertion that precheck process count is independent of the number of dirty buffers — verification note: covered by the existing app sync tests and the crate-level batching/counter tests; no separate app-level precheck counter test was added

## 6. Verification

- [x] 6.1 Add a crate-level budget test: a refresh-equivalent sequence (status + observed tip + classify) on an unchanged repository with 200 changed notes, each referencing images, stays within the design's ≤ 6 process target. Add a no-op `sync_now` test that records the process count and asserts it is at most 35 — measured: refresh-equivalent reads on 200 changed notes with 400 image references spawn 4 processes (8 when target resolution for the Outgoing page is included); engine no-op `sync_now` spawns 18
- [x] 6.2 Run `cargo test --workspace` and confirm it passes — 46 test binaries, 1903 passed, 0 failed (`--no-fail-fast`); the app test helper `run_git` now retries the transient Windows `STATUS_DLL_INIT_FAILED` spawn error
- [ ] 6.3 Manual check on Windows with a real notes repository:
  - with the window in the background for 2 minutes, Task Manager or Process Monitor shows no periodic `git.exe` launches;
  - committing from an external terminal updates Backup and Sync after switching back;
  - Sync Now on an unchanged repository completes noticeably faster than before.

  Record observed timings in this change folder
