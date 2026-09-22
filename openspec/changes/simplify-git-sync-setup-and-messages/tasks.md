## 1. Commit-message context in markion-git-sync

- [x] 1.1 Add `chrono` (default-features off, `clock` on) to `crates/git-sync/Cargo.toml` and verify `cargo build -p markion-git-sync` succeeds with no gpui dependency introduced
- [x] 1.2 Rework `default_commit_message` in `crates/git-sync/src/policy.rs` to take `(paths: &[PathBuf], timestamp: SystemTime)` and render `Sync notes YYYY-MM-DD HH:MM: a.md, b.md, +3 more` (file names only, at most three names, `+N more` tail, ~72-char subject cap, local time with UTC fallback); verify new unit tests cover single file, many files, subject-cap truncation, and the fallback path
- [x] 1.3 Rework `RepositoryPolicy::render_message` to take `(paths, device_name, timestamp)` and substitute `{count}`, `{date}`, `{time}`, `{files}` in custom templates (empty template delegates to the new default; device-name suffix unchanged); verify unit tests cover each placeholder, combined templates, and the empty-template delegation
- [x] 1.4 Update the crate's public exports in `crates/git-sync/src/lib.rs` and fix any crate tests still calling the old signatures; verify `cargo test -p markion-git-sync` passes

## 2. Adoption eligibility predicate in markion-git-sync

- [x] 2.1 Add the pure predicate `repository_adoptable(workspace_root, identity, capabilities, tracked_paths, target)` to `crates/git-sync/src/setup.rs` encoding all five eligibility criteria (workspace is worktree root, HEAD exists, write sync supported, origin target resolvable, no unrelated tracked content); verify unit tests exercise each criterion's pass and fail case plus one fully-eligible combination
- [x] 2.2 Export the predicate from `crates/git-sync/src/lib.rs` and verify `cargo test -p markion-git-sync` passes with the new export

## 3. Localized adoption and template chrome

- [x] 3.1 Add `GitMsg` entries in `src/i18n/git.rs` for the adoption status notification (parameterized with the repository name/path) and for message-template help listing `{date}`, `{time}`, `{files}`, `{count}`, translated into every supported language; verify the app compiles (the i18n compile-time completeness check must pass)
- [x] 3.2 Surface the expanded placeholder help on the message-template settings field in the sync settings form (`src/app/git_panel.rs`); verify the settings surface renders the new help in at least the default and one other language via existing settings/UI tests

## 4. App wiring: adopt-and-sync at the unconfigured entry points

- [x] 4.1 Add the background adoption probe in `src/app/git_sync.rs` (background spawn, generation counter and `git_ui.running`/`settings` guards mirroring the existing onboarding probe): discover + review the repository, apply `repository_adoptable`, and on success upsert `default_notes_policy` with `background_fetch` from `git_preferences.background_check`, post the localized adoption notification, and proceed directly to `start_git_sync`; verify a new app-level test shows an eligible workspace going from no policy to a saved policy and a running Sync Now without the onboarding surface
- [x] 4.2 Route both sync-intent entry points through the probe — `sync_now`'s `SetupExistingRepository` branch and `setup_git_sync`/`BackupSyncAction::TurnOn` — while ineligible workspaces keep opening `open_git_onboarding` unchanged; verify app-level tests cover the fallback cases (workspace is a subdirectory, unrelated tracked content, missing remote, no commits) alongside the existing `sync_now_without_markion_policy_starts_existing_repository_setup` behavior
- [x] 4.3 Pass `&paths` and `SystemTime::now()` into `policy.render_message` at the `SyncPlan` build site in `run_git_operation` and update any app tests asserting the old `Sync notes: N files` text; verify `cargo test` for the root package passes

## 5. End-to-end verification

- [x] 5.1 Run `cargo test --workspace` and `openspec validate simplify-git-sync-setup-and-messages`; both must pass
- [ ] 5.2 Manual smoke check on a scratch clone: open a dedicated notes Git repository as the workspace, trigger Sync Now, and confirm no setup dialog appears, the adoption notification shows, the sync completes, and `git log` shows the new compact date-time-and-files message; then repeat with a mixed-content repository and confirm the advanced setup dialog still appears

## 6. Adoption eligibility relaxation (post-review fix)

- [x] 6.1 Add `tracked_path_is_notes_like` to `crates/git-sync/src/policy.rs` (notes/text/image OR hidden dot-path OR furniture name LICENSE/COPYING/NOTICE/README), export it from `lib.rs`, and use it in `repository_adoptable` and the app's `has_unrelated_tracked_content` so real-world notes repositories with `.gitignore`/`.obsidian`/`LICENSE` become adoptable; verify unit tests cover furniture tolerance and visible non-notes rejection
- [x] 6.2 Add an integration test where a clone tracking `.gitignore`, `.obsidian/app.json`, and `LICENSE` is still adopted while a `Cargo.toml` repository is not; verify `cargo test -p markion-git-sync` passes
