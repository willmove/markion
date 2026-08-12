## Context

The file-tree scan lives in `src/storage/file_tree.rs`. `FileTree::scan(root)` delegates to `collect_file_tree_entries`, which calls `should_skip_file_tree_path(path)` for every directory entry (line ~267). That single predicate today conflates two distinct concerns:

1. An always-on, hard-coded list of build/dependency/VCS directories (`target`, `node_modules`, `.git`, `.venv`, `__pycache__`, …) excluded by exact name.
2. A dotfile-directory check — but it requires `path.is_dir()`, so dotfile **files** (e.g. `.secret.md`) are *not* filtered and currently leak into the tree. There is no Windows `FILE_ATTRIBUTE_HIDDEN` check anywhere in `src`.

The scan pre-prunes folders that contain no Markdown descendant, and it pre-filters non-Markdown files, so the resulting `FileTree` is already a pruned flat depth-tagged `Vec`. The scan runs off the UI thread on `cx.background_executor()` (`src/app/application.rs` `schedule_file_tree_scan`), and the resulting `FileTree` is written back into `app.file_tree`. Preferences are loaded at startup into `MarkionApp`, snapshotted to `AppPreferences` for save, and persisted to `config.toml` via the serde-facing `PreferencesFile` in `src/storage/preferences.rs` (kept separate from `model` so `model` stays dependency-free). See `proposal.md` for the motivation.

## Goals / Non-Goals

**Goals:**
- Give "hidden" one consistent, OS-aligned definition for files *and* folders, controllable by a persisted preference.
- Keep the change small and pattern-consistent with the existing boolean preferences (`sync_scroll`, `preview_adaptive_width`): model field → serde layer → app state → toggle handler → Preferences row → i18n keys.
- Preserve every documented invariant: bounded rows per frame, Markdown-only filtering, untouched Markdown-derived caches, no `gpui` dependency in `crates/*`.

**Non-Goals:**
- Not redesigning the scan into a lazy/streaming tree, and not adopting the `ignore` crate / `.gitignore` support.
- Not making the build/dependency noise list configurable, and not removing it.
- Not per-workspace or per-entry visibility — the preference is global.

## Decisions

### Decision 1 — "Hidden" means OS-hidden, not "on the noise list"

The toggle governs **only** the OS-hidden layer: a leading `.` in the file name on every platform, plus the Windows `FILE_ATTRIBUTE_HIDDEN` attribute on Windows. The existing hard-coded noise list stays a separate, always-on filter layer that the toggle cannot reveal.

Rationale: this matches what "show hidden files/folders" means in every native file explorer and in VS Code's default `files.exclude` semantics — users expect `.secret.md` or a Windows-hidden `Thumbs.db` to appear, they do not expect `target/` or `node_modules/` to flood the tree. It also gives us a principled place (the noise list) to keep excluding things regardless of the preference.
*Alternative considered:* treat the toggle as "reveal everything including the noise list". Rejected — it conflates two concerns and produces an overwhelmingly noisy tree, and `.git` exposure would invite accidental edits.

### Decision 2 — Default is off, and dotfile files become hidden too

Default `false` preserves today's hide-the-dotfiles experience. As a deliberate consistency fix, the OS-hidden check is extended to dotfile **files** (not just directories), so `.secret.md` — which currently leaks through — will be hidden under the default and only revealed when the user opts in. This makes files and folders behave identically.

Rationale: the current file/dir asymmetry is a latent bug; "hidden" should mean the same thing for both. Users who relied on seeing dotfile Markdown files can flip the toggle.
*Alternative considered:* keep dotfile files visible by default to avoid any behavior change. Rejected — it leaves the asymmetry in place and makes the spec harder to reason about ("hidden folders yes, hidden files no").

### Decision 3 — Thread the flag via `scan_with_options(root, show_hidden)`

Add `pub fn scan_with_options(root: &Path, show_hidden: bool) -> FileTree` and keep the existing `pub fn scan(root: &Path) -> FileTree` as a thin wrapper that calls `scan_with_options(root, false)`, so the two other `FileTree::scan` call sites (`src/app/workspace.rs` lines ~650, ~670) and any tests keep compiling unchanged. The flag is passed down to `collect_file_tree_entries` and into the skip predicate.

*Alternative considered:* a `ScanOptions { show_hidden: bool }` struct for extensibility. Rejected as YAGNI — there is exactly one option today and a struct adds ceremony without a second consumer. Easy to promote later.

### Decision 4 — Split `should_skip_file_tree_path` into two layers

Refactor the predicate into two functions:
- `is_always_excluded(path)` — the existing hard-coded noise list (unchanged behavior), called unconditionally.
- `is_hidden_entry(path, show_hidden)` — returns `true` when the entry is OS-hidden (dotfile name, or Windows hidden attribute) **and** `show_hidden` is `false`. When `show_hidden` is `true` it returns `false` so hidden entries pass through (still subject to `is_always_excluded` and the Markdown-only filter).

The scan skips an entry when *either* layer says skip. This keeps the noise-list logic byte-for-byte identical and isolates the new behavior.

### Decision 5 — Toggling re-scans the tree rather than filtering in place

The toggle handler flips `app.show_hidden_files`, persists, then calls `self.refresh_file_tree(cx)` — the existing path that schedules a background scan. We deliberately do *not* keep hidden entries in memory and filter them at render time.

Rationale: the skip predicate runs *during* the scan, so a hidden folder's entire subtree is naturally omitted (the recursion never enters it) and naturally reappears — folder plus its children — when the preference is on. Note the scan does **not** content-prune folders: empty folders and folders containing only non-text files are kept as nesting rows, so visibility is governed entirely by the skip predicate, not by descendant content. An in-place render-time filter would instead have to keep hidden entries in memory, re-derive visibility on every render, and still decide per-hidden-folder whether to enter it — re-scanning keeps the `FileTree` shape authoritative and is simpler. Re-scanning is cheap (background-threaded `read_dir`, user-initiated, infrequent) and reuses the established refresh path. The collapse state is already re-anchored after every scan by `update_file_tree_collapse_state_from_scan`, so expand/collapse state survives the rescan.

### Decision 6 — Windows attribute check is `cfg(windows)`-gated, lazy, and failure-tolerant

On Windows, `is_hidden_entry` additionally calls `std::fs::metadata(path)` and checks `MetadataExt::file_attributes() & 0x2` (`FILE_ATTRIBUTE_HIDDEN`). This `metadata` call is gated behind `cfg(windows)` so non-Windows builds pay nothing. It is best-effort: if the `metadata` call fails (e.g. broken symlink, permission), the entry is treated as *not* hidden-by-attribute (the dotfile check still applies independently), so a single unreadable entry never aborts the whole scan.

## Data flow / caching

1. Startup: `MarkionApp::new` loads `config.toml` into `AppPreferences` (new `show_hidden_files` field, default `false`) and copies it onto the app struct.
2. Scan: `schedule_file_tree_scan` reads `app.show_hidden_files` and calls `FileTree::scan_with_options(root, show_hidden)` on the background executor; the resulting `FileTree` replaces `app.file_tree`, then `update_file_tree_collapse_state_from_scan` re-anchors `collapsed_tree_paths`.
3. Toggle: `toggle_show_hidden_files` flips the field, sets a localized status, calls `persist_preferences()`, then calls `refresh_file_tree(cx)` → step 2.
4. Save: `current_preferences()` snapshots the field back into `AppPreferences`; `save_app_preferences` writes `config.toml`.

No Markdown-derived cache (preview blocks, outline, stats), no syntax-highlighting memoization, and no editor text handle is touched by this flow — the file tree is a separate, per-scan structure, not per-keystroke derived state. The bounded-rows-per-frame invariant is preserved because revealing hidden entries can only add rows that already exist on disk; it does not change the rendering loop.

## Risks / Trade-offs

- **[Risk] Dotfile Markdown files that users currently see disappear by default.** → *Mitigation:* this is the intended consistency fix; the toggle restores them in one click, and the spec calls the default out explicitly so it is a documented, testable behavior rather than a silent regression.
- **[Risk] Extra `std::fs::metadata` syscall per entry on Windows.** → *Mitigation:* the scan already does `read_dir` + per-entry `is_dir`/extension work; one more `metadata` read on Windows-only is negligible for a user-initiated, background scan. Non-Windows builds are unaffected.
- **[Risk] Re-scan on every toggle could feel slow on huge workspaces.** → *Accepted:* toggling is rare and user-initiated, and the scan is already background-threaded; the alternative (in-place visibility filter) would require reworking folder pruning and is out of scope.
- **[Trade-off] The noise list is not configurable.** → *Accepted:* users wanting `target/` or `node_modules/` in the tree are out of luck for now; this is a deliberate Non-goal and can be revisited as a separate change.

## Migration Plan

- No on-disk migration is required. `PreferencesFile` is `#[serde(default)]`, and the new field uses `deserialize_bool_or_false`, so existing `config.toml` files without the key load as `false` (today's behavior) and round-trip cleanly once saved.
- No spec migration: the `workspace` Markdown-only-filter requirement is unchanged; this change *adds* a new requirement rather than modifying it.
- Rollback: revert the code; persisted `show_hidden_files` keys in `config.toml` are simply ignored by older builds (serde default), so downgrading is safe.

## Open Questions

- None. The definition of "hidden", the default, and the scan strategy are all decided above; the i18n label wording is a tasks-level detail resolved during implementation.
