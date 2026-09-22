# Proposal: simplify-git-sync-setup-and-messages

## Why

Users whose notes folder is already an active Git repository (cloned or pushed from elsewhere) still have to walk the Backup-and-Sync setup dialog before their first sync, even though the repository, remote, and branch are already configured and working. Separately, the automatic commit message (`Sync notes: N files`) records neither when the sync happened nor which notes were involved, making history hard to scan.

## What Changes

- **Automatic adoption of healthy existing repositories.** When the user first invokes Sync Now for a workspace with no saved sync policy and the workspace root is itself a suitable repository — it has at least one commit, a configured origin with a resolvable fetch/push target, supports write sync, and its tracked content is notes-like — Markion records the repository policy silently and runs the requested sync immediately, with no setup dialog. A status notification states that the existing repository was connected, and the policy stays editable/removable in settings exactly like a manually configured one.
- **Fallback unchanged.** Workspaces that do not meet the adoption criteria (workspace is a subdirectory of a larger repository, unrelated/mixed tracked content, missing remote, no commits yet, or read-only repository) keep the current contextual onboarding route with advanced review.
- **Adoption is Git-read-only.** Adoption performs repository discovery and policy-file creation only; it never changes remotes, branches, staging, or commit history, and it never requires a network round-trip before the first sync proceeds.
- **Richer default commit message.** The automatic sync commit message becomes one compact line carrying local date-time and bounded file names, for example `Sync notes 2026-09-18 14:32: a.md, journal.md, +3 more`. The list is capped (few names plus an `+N more` tail) so the subject stays short.
- **Template placeholders.** The user-editable message template gains `{date}`, `{time}`, and `{files}` placeholders alongside the existing `{count}`, with localized in-app help listing them.
- Non-goals: no adoption triggered merely by opening a workspace (adoption happens on the user's first sync request); no network liveness check as a prerequisite for adoption; the generated default message stays machine-generated English (repo-portable) rather than localized; version-history draft messages and the one-time initial `Start notes` commit are unchanged; no new policy schema version.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `git-workspace`: adds a requirement that a healthy, dedicated existing repository at the workspace root is adopted for sync without manual setup, with explicit eligibility criteria and fallback to the existing onboarding routes.
- `git-sync`: adds a requirement that automatic sync commit messages carry local date-time and bounded file-name context in a compact single line, and that message templates expose date/time/files placeholders.
- `ui-i18n`: adds a requirement covering the localized adoption status notification and the expanded message-template placeholder help.

Note: `git-workspace` and `git-sync` are capabilities introduced by the completed, not-yet-archived `add-git-workspace-sync` change; this change's deltas are written against that baseline and must be archived after it.

## Impact

- `crates/git-sync/src/policy.rs`: `default_commit_message` and `RepositoryPolicy::render_message` change signature (changed paths plus timestamp instead of a count) and gain placeholder support; crate tests updated.
- `crates/git-sync/Cargo.toml`: add a small GUI-free datetime dependency for local-time formatting (recommended: `chrono` with default features off plus `clock`), with UTC fallback when the local offset is unavailable.
- `src/app/git_sync.rs`: `sync_now`'s `SetupExistingRepository` route gains a background eligibility probe that either upserts the default notes policy (adoption, reusing `default_notes_policy` with `background_fetch` from the global preference) and proceeds to sync, or falls back to `open_git_onboarding`.
- `src/i18n/git.rs` (+ generated i18n surface): new localized strings for the adoption notification and template-placeholder help in all supported languages.
- Tests: crate-level message-format tests (including truncation and placeholder cases), app-level tests for the eligibility predicate and the `sync_now` adoption/fallback routing, mirroring the existing `sync_now_without_markion_policy_starts_existing_repository_setup` integration test.
- Invariants preserved: adoption probing and message formatting run in background executors, never on the typing/render path; no derived Markdown caches are recomputed; `markion-git-sync` stays gpui-free; no policy schema change (existing persisted policies keep working).
