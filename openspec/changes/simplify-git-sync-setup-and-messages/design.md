## Context

The Backup-and-Sync feature ships from `add-git-workspace-sync` (complete, not yet archived; this change builds on its code and spec deltas). Two friction points drive this change:

- `sync_now` resolves the workspace against saved policies; with none, it routes to `open_git_onboarding` (`src/app/git_sync.rs`), and the unconfigured sync center's primary action (`BackupSyncAction::TurnOn` → `setup_git_sync`) opens the same dialog. A user whose notes folder is already a working Git repository must confirm a setup surface that adds no information.
- Automatic sync commit messages come from `RepositoryPolicy::render_message(changed_files, device_name)` → `default_commit_message(count)` = `Sync notes: N files` (`crates/git-sync/src/policy.rs`). The plan build site (`run_git_operation` in `src/app/git_sync.rs`) already has the exact changed-path list, so richer context costs nothing new to gather.

Constraints: `markion-git-sync` must stay gpui-free; no Git work on the typing/render path; policy file schema is versioned (`POLICY_SCHEMA_VERSION`) and must not change; user-facing strings go through `src/i18n.rs`.

## Goals / Non-Goals

**Goals:**

- First sync request in an eligible workspace adopts the existing repository with zero dialogs and immediately runs the requested sync.
- Adoption is a read-only probe plus one policy-file write; it is announced and reversible.
- Automatic commit messages carry local date-time and bounded file names on one compact line; templates gain `{date}`/`{time}`/`{files}` placeholders.

**Non-Goals:**

- No adoption on mere workspace open (no discovery subprocess per open; no surprise background fetch enablement).
- No network liveness ("does pull work right now?") precondition for adoption — Sync Now itself performs the fetch with full offline/authentication handling.
- No localization of the generated commit message (repository content, not UI chrome; stays English like the existing `Sync notes` prefix and the one-time `Start notes` commit).
- No change to version-history drafts, `Start notes`, or the policy schema.

## Decisions

### 1. Adoption triggers on user sync intent, at both unconfigured entry points

Hook the eligibility probe where `open_git_onboarding` is currently reached from user intent: `sync_now`'s `SetupExistingRepository` route (sync-after-setup = true) and `setup_git_sync`/`BackupSyncAction::TurnOn` (sync-after-setup = true as well — "turn on backup and sync" implies the first sync). When eligible, adopt and run Sync Now directly; when ineligible, fall back to the existing dialog untouched. Explicit advanced-setup surfaces keep opening the dialog.

*Alternative rejected:* adopting on workspace open. It would run `git discover` subprocesses on every open/switch, enable background fetch before the user expressed any intent, and can surprise users who deliberately keep a repo unconnected. Adoption only on explicit sync intent keeps the action user-initiated.

### 2. Eligibility predicate lives in `markion-git-sync`, wired by the app

Add a pure predicate in `crates/git-sync/src/setup.rs`, e.g. `repository_adoptable(workspace_root, identity, capabilities, tracked_paths, target) -> bool`, encoding all five spec criteria:

1. canonical workspace root equals the discovered worktree root (subdirectory stays on the advanced route, as the baseline spec requires);
2. `state.head` is `Some` (repo has commits — the structural proxy for "active");
3. `capabilities.supports_write_sync()`;
4. `connect_existing`-style review yields `Some(target)` — an origin resolving to exactly one fetch/push destination for the current branch (the structural proxy for "pull/push works"; upstream ambiguity or a missing remote falls back to setup);
5. no unrelated tracked content — via `tracked_path_is_notes_like`: a tracked path is notes-like when it classifies as notes/text/image, is a hidden dot-path (`.gitignore`, `.obsidian/…`), or is standard repository furniture (`LICENSE`, `COPYING`, `NOTICE`, `README`); visible non-notes files such as source files still count as unrelated (same rule as `repository_onboarding_requires_advanced` today, which was updated to share the helper).

> Post-review correction: the first implementation classified furniture (`LICENSE`, `.gitignore`, `.obsidian/*.json`) as unrelated content, which blocked adoption for essentially every real-world notes repository and routed users back to the setup dialog. Discovered in user testing; the relaxation above and the shared helper fix it for both adoption and the onboarding advanced gate.

The app runs discovery + `connect_existing` in a background spawn (mirroring the existing onboarding probe's generation-counter pattern to ignore stale probes after workspace switches), applies the predicate, and on success upserts `default_notes_policy(identity, target, git_preferences.background_check)` — the identical policy a manual `UseCurrentFolder` confirmation produces, including the background-fetch default already sourced from the global preference at `submit` time.

*Why a pure predicate:* crate-level unit tests cover each criterion without GPUI or window state; the app layer only orchestrates.

### 3. Commit message rendering moves from a count to (paths, timestamp)

In `policy.rs`:

- `default_commit_message(paths: &[PathBuf], timestamp: SystemTime) -> String` → `Sync notes YYYY-MM-DD HH:MM: a.md, b.md, +3 more`.
- `RepositoryPolicy::render_message(&self, paths: &[PathBuf], device_name: Option<&str>, timestamp: SystemTime) -> String` — template branch substitutes `{count}`, `{date}`, `{time}`, `{files}`; default branch (empty template) calls the new default; device-name suffix behavior unchanged.

File-list rules: take `Path::file_name()` of each path (names, not full paths, per compactness), paths are already sorted at the call site; show at most 3 names; any remainder becomes `+N more`; enforce a compact subject cap (~72 chars) by dropping whole names into the tail (worst case: `Sync notes <dt>: +N more`).

The call site in `run_git_operation` passes `&paths` and `SystemTime::now()` when building `SyncPlan`. The timestamp is plan-build time, not commit-instant: `SyncPlan.message` is the journaled/recoverable intent, and rendering inside the engine would couple it to execution timing for cosmetic gain.

### 4. Local time via `chrono` in the member crate

Add `chrono` (default-features off, `clock` on) to `crates/git-sync/Cargo.toml` and format `YYYY-MM-DD HH:MM` in the local zone, falling back to UTC when the offset is unavailable (satisfying the spec's fallback scenario).

*Alternative rejected:* the `time` crate — its `local-offset` feature is unreliable in multithreaded Unix processes (returns `Err` unless the `unsound_local_offset` escape hatch is enabled), which this GPUI app always is. Hand-rolled civil-date math from `SystemTime` was also rejected: timezone rules (DST, zone offsets) are not worth reimplementing. `chrono` is GUI-free, so the member-crate invariant holds; it is not on the typing path, so no `[profile.dev.package]` override is warranted.

### 5. Announcement and help through i18n

New `GitMsg` entries in `src/i18n/git.rs` (all supported languages): an adoption status notification (parameterized with the repository name/path) and expanded template-placeholder help listing `{date}` `{time}` `{files}` `{count}` for the settings field that currently documents only `{count}`. The commit message text itself remains engine-generated English, as decided above.

## Risks / Trade-offs

- [Auto-adoption acts without an explicit consent click] → It only fires on an explicit sync-intent action, performs no Git mutation, is announced via status, and the resulting connection is removable in settings — same artifact a manual confirmation would have produced.
- [An adopted repo later accumulates unrelated content] → Unchanged existing policy behavior already handles it: out-of-scope tracked changes and unclassifiable new files become `Attention` decisions that block automatic commits until reviewed. Adoption does not widen what a sync may stage.
- [Message list could still be long or repeat identical names from different directories] → Three-name cap plus `+N more` tail and the subject-length cap bound the line; name-only listing is the accepted compactness trade-off (templates can use explicit text if paths matter to a user).
- [`chrono` adds a dependency to the workspace graph] → Widely maintained, GUI-free, small feature set enabled; lockfile impact only, no new invariant.
- [Timestamp rendered at plan build drifts from the actual commit second if fetch/merge is slow] → Minute precision makes this immaterial in practice; keeping the message in the journaled plan preserves recovery semantics.

## Migration Plan

None required. Empty-template policies render the new default immediately; custom templates keep working (`{count}` semantics unchanged, new placeholders purely additive). Rollback is a plain revert — any policy written by adoption is indistinguishable from a manual one and remains valid.

## Open Questions

- Exact wording of the adoption notification and placeholder help per language — resolved during implementation with the existing `src/i18n/git.rs` conventions.
