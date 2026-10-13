## Why

Backup and Sync keeps Git busy even when nothing is happening. A timer refreshes the Git details every 5 seconds whether or not the panel is open, the window is focused, or anything changed. Every repository status read also probes repository capabilities with seven extra Git processes plus a walk of the entire worktree. Per-path checks spawn one or two Git processes for each changed path, ignored candidate and image reference. On the reference Windows machine one `git` spawn costs about 230 ms, so a routine refresh takes seconds of continuous background Git activity, and a no-op Sync Now starts roughly 70–90 Git processes. This drains laptop batteries, churns antivirus scanning, and makes Sync Now feel slow for reasons the user cannot see.

## What Changes

- **Event-driven details refresh**: the fixed 5-second details poll is replaced by refreshes triggered by:
  - worktree/control-file change notifications from the existing file watcher;
  - window activation;
  - save and Git-operation completion;
  - workspace switches and opening Backup and Sync.

  A slow fallback poll runs only while the window is active. Bursts of triggers coalesce into one refresh.
- **Cheaper repository probes**:
  - repository discovery uses one Git process;
  - capability detection runs once per repository instance (once per write operation) instead of inside every status read;
  - nested-repository detection uses Git's own status/index information instead of walking the whole worktree;
  - recovery assessment reuses the status it was given instead of reading status again, and skips work when no checkpoint exists.
- **Batched per-path queries**:
  - ignore and tracked checks for buffers and attachment references use one batched Git invocation per refresh/operation;
  - incoming-path validation uses one diff listing instead of two tree lookups per incoming path;
  - the redundant ignore check on status-reported untracked paths (status already excludes ignored files) is removed.
- **Bounded UI-thread work**:
  - a refresh copies the text of dirty documents only, not every open document;
  - content fingerprints are cached by file size/mtime (on-disk files) and by document instance/version (dirty buffers), so unchanged files are not re-hashed on each refresh.
- Non-goals:
  - changing sync semantics, the safety preconditions, or what the status projection reports;
  - network timeouts and offline detection (separate change);
  - skipping the no-op push;
  - moving Git work into a long-lived Git library binding.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `git-sync`: adds a requirement that repository status refresh is change-driven and has bounded Git process cost that does not grow with the number of changed paths or referenced attachments, while still reflecting external repository changes promptly.

## Impact

- `crates/git-sync`:
  - `repository.rs`: single-process discovery, memoized capability probe, status-derived nested-repository detection, batched `ignored_paths` / `tracked_paths_among` helpers;
  - `engine.rs`: incoming-path validation from one `diff --raw`, removal of per-path `check-ignore` in preflight;
  - `resources.rs`: batched attachment inspection;
  - `conflict.rs`: `assess` accepts an existing status;
  - `process.rs`: an optional invocation counter for tests.
- `src/app/git_panel.rs`: refresh triggers, coalescing, dirty-only text capture, fingerprint cache.
- `src/app/application.rs`: the file watcher also signals Git refreshes.
- `src/app/git_sync.rs`: batched buffer checks.
- `src/app/bootstrap.rs`: arms the new refresh scheduling instead of the 5-second poll.
- Invariants:
  - all Git and filesystem work stays on background executors;
  - no derived Markdown state is recomputed;
  - fingerprints of dirty buffers are cached per document version, consistent with the cached-per-version invariant;
  - `markion-git-sync` stays gpui-free.
