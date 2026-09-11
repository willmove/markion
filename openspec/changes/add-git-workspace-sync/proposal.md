## Why

Markion users need to know that personal Markdown notes and attachments are safely backed up and consistent across computers without first learning Git. The implemented synchronization engine is recoverable and complete, but its persistent Sync sidebar, duplicate entry points, repository terminology, and equally prominent low-level commands expose the transport model during ordinary note-taking; the feature must present a simple backup-and-sync experience while retaining Git power and diagnostics as an advanced layer.

## What Changes

- Add a workspace-bound **Sync Now** action: save eligible named documents, create a local commit with an automatic message, fetch, fast-forward or merge, and push a fixed commit to one configured remote branch. After initial scope approval, ordinary sync requires no repeated file selection, commit-message entry, or confirmation.
- Project the engine's independent persistence and repository dimensions into one honest consumer-facing state such as not configured, pending locally, incoming updates, synchronized with a confirmation time, offline with local work retained, authentication needed, conflict, or uncertain delivery. Never describe a local-only commit as a remote backup or show an unknown remote state as synchronized.
- Explain that synchronization propagates note edits and deletions and that Version History is the recovery path; do not imply that the synchronized destination is an immutable archival backup.
- Support cloning an existing or empty remote, connecting an existing local repository, and initializing a notes folder against an empty remote. Detect the remote default branch and existing upstream; explain whole-repository effects when the workspace is a subdirectory.
- Present setup as one contextual **Backup and Sync** path at a time: reuse a suitable dedicated notes repository, enable the current notes folder, or obtain notes from an existing sync address. Keep alternative routes, branch, author, scope, repository history, and transport details in advanced review instead of displaying three Git workflows as equal first-use choices. Route mixed repositories and workspace subdirectories to explicit advanced review rather than silently broadening the ordinary notes scope.
- Persist an explicit personal-notes policy: approved paths for tracked changes and approved directories/types for new notes and attachments. Show all Git changes independently of the document tree; unexpected files and changes outside the policy require attention. File selection limits a new commit, not the branch history transferred.
- **BREAKING UI:** Replace the persistent Sync sidebar tab and duplicate Sync/Sync Now controls with one stateful **Backup and Sync** entry in workspace chrome. Activating it opens a transient sync center with current state, destination, last confirmed time, one context-sensitive primary action, human-readable pending/incoming counts, history, and settings.
- Keep complete changes, outgoing history, diffs, branch/remote diagnostics, Commit Locally, Check Remote, Pull Updates, and Push Commits available under an explicitly advanced Git section. Put per-file version history in the file context as well as the sync center instead of requiring users to navigate a repository dashboard.
- Coordinate Git worktree mutations with every app write into that repository, including already-running autosaves; reload only affected documents and images through checked lifecycle boundaries.
- Provide in-app text and image/binary conflict resolution in a focused, adequately sized surface using note-version language by default, with durable drafts, explicit completion/abort, and restart reconciliation. Keep successful local commits when networking fails and verify uncertain push outcomes before retrying.
- Use system Git with HTTPS credential helpers and SSH-agent integration; expose executable detection and repository connection settings without storing credentials in notes or session files.
- Let explicit foreground setup and sync operations use the operating system's secure credential prompt/helper or SSH agent, then offer a clear retry after authentication is completed. Background checks remain strictly noninteractive.
- Permit opt-in background checking for other-device updates on the active connected workspace only, with copy that explicitly says it does not automatically upload or apply changes. Background commit, merge, and push are deferred; the first release is a complete user-triggered sync loop.
- Migrate a persisted legacy Sync-sidebar selection to the Files view without altering repository connections, pending recovery, or notes.

Non-goals: a general Git client; managed cloud storage or remote-host sign-in/repository creation APIs; hiding material whole-repository risk from advanced mixed-repository users; branch switching/creation UI beyond initial setup; rebase, force-push, stash management, history rewriting, whole-repository rollback, automatic background write synchronization, recursive submodule sync, LFS transfer management, or write support for linked worktrees, bare, sparse, partial, and shallow repositories.

## Capabilities

### New Capabilities

- `git-workspace`: System Git discovery, contextual notes-oriented onboarding and capability checks, explicit remote/branch binding, credentials, identity, and approved personal-notes scope.
- `git-sync`: Consumer-facing backup/sync states, one-click and advanced operations, complete repository status, bounded execution, exact-target push, offline behavior, and optional background fetch.
- `git-conflict-recovery`: Note-version conflict sessions, safe resolution and abort, durable operation checkpoints, and restart/uncertain-result reconciliation.

### Modified Capabilities

- `workspace`: Bind sync to repository identity across workspace/tab changes; coordinate file-tree mutations, preserve foreign/untitled buffers, and migrate the removed persistent Sync view safely.
- `reliable-file-persistence`: Add a repository write barrier covering in-flight saves, external reloads, and Git worktree updates without weakening atomic-save/recovery guarantees.
- `document-resources`: Validate attachment participation and portability, coordinate resource writes, and invalidate images changed by Git without reparsing unchanged Markdown.
- `chrome-platform`: Provide a single stateful Backup and Sync entry, transient simple sync center, file-context history, and an advanced Git layer while retaining active-document branch semantics.
- `theme-preferences`: Separate ordinary Backup and Sync settings from advanced Git executable/connection controls, clarify background-check behavior, and persist repository policies independently of recent-workspace eviction.
- `ui-i18n`: Localize consumer-facing backup/sync states separately from advanced Git setup, errors, credentials, conflict resolution, and recovery surfaces in all supported languages.
- `crate-architecture`: Introduce a GUI-free `markion-git-sync` member and root-app adapters with cached status and no Git work on the typing/render path.

## Impact

- New `crates/git-sync/` member (package `markion-git-sync`) and root-app sync controller/UI modules. System Git is an optional runtime prerequisite for this feature, not for opening or editing documents; no bundled Git distribution is introduced.
- Integrations: `src/app/root_view.rs`, `git_panel.rs`, `git_conflicts.rs`, status/workspace/menu/bootstrap code, file context menus, `application.rs`, `documents.rs`, resource/export write entry points, `src/lib.rs` checked save/reload boundaries, `src/model.rs`, `src/storage/`, `src/paths.rs`, shortcuts, and `src/i18n.rs`.
- Local persistence: additive global settings, a versioned repository-policy file separate from `session.toml`, and operation/draft storage outside repositories. Git retains ownership of remotes, upstream, and repository author configuration; credential helpers own saved secrets.
- Existing completed but unarchived `add-recent-workspace-switcher` and `configure-silent-save-in-preferences` behavior is the integration baseline. This change does not revert retained dirty tabs or recovery-only autosave, and does not directly edit stable specs.
- Preserve per-document-version `Arc` Markdown caches, syntax-highlight memoization, cached text handles, bounded tree rendering, and GUI-free member crates. Git status changes alone never increment document versions.
- Validation: state-projection and legacy-view migration tests, narrow-layout/keyboard/non-color UI checks, real bare-remote/two-clone integration tests, save/network interruption fault tests, three-platform setup/credential/conflict smoke checks, and `cargo test --workspace` before delivery. No external repository is modified as part of creating this proposal.
