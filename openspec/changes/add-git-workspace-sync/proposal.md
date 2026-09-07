## Why

Markion users need to keep personal Markdown notes and their attachments consistent across computers without leaving the editor or manually coordinating Git commands. The app already has workspace sessions, durable saves, recovery snapshots, and a branch indicator, but no repository connection or synchronization workflow; adding a complete, recoverable manual sync loop makes Git practical as a personal notes transport.

## What Changes

- Add a workspace-bound **Sync Now** action: save eligible named documents, create a local commit with an automatic message, fetch, fast-forward or merge, and push a fixed commit to one configured remote branch. After initial scope approval, ordinary sync requires no repeated file selection, commit-message entry, or confirmation.
- Support cloning an existing or empty remote, connecting an existing local repository, and initializing a notes folder against an empty remote. Detect the remote default branch and existing upstream; explain whole-repository effects when the workspace is a subdirectory.
- Persist an explicit personal-notes policy: approved paths for tracked changes and approved directories/types for new notes and attachments. Show all Git changes independently of the document tree; unexpected files and changes outside the policy require attention. File selection limits a new commit, not the branch history transferred.
- Add a compact workspace sync entry, a Sync sidebar, optional change/history details, and secondary Commit Locally, Check Remote, Pull Updates, and Push Commits actions. Distinguish unsaved buffers, uncommitted files, outgoing commits, incoming commits, and timestamped confirmation.
- Coordinate Git worktree mutations with every app write into that repository, including already-running autosaves; reload only affected documents and images through checked lifecycle boundaries.
- Provide in-app text and image/binary conflict resolution, durable drafts, explicit merge completion/abort, and restart reconciliation. Keep successful local commits when networking fails and verify uncertain push outcomes before retrying.
- Use system Git with HTTPS credential helpers and SSH-agent integration; expose executable detection and repository connection settings without storing credentials in notes or session files.
- Permit opt-in background fetch for the active connected workspace only. Background commit, merge, and push are deferred; the first release is a complete user-triggered sync loop.

Non-goals: a general Git client; remote-host repository creation APIs; branch switching/creation UI beyond initial setup; rebase, force-push, stash management, history rewriting, whole-repository rollback, automatic background write synchronization, recursive submodule sync, LFS transfer management, or write support for linked worktrees, bare, sparse, partial, and shallow repositories.

## Capabilities

### New Capabilities

- `git-workspace`: System Git discovery, repository onboarding and capability checks, explicit remote/branch binding, credentials, identity, and approved personal-notes scope.
- `git-sync`: One-click and secondary operations, complete repository status, bounded execution, exact-target push, offline behavior, and optional background fetch.
- `git-conflict-recovery`: Conflict sessions, safe resolution and abort, durable operation checkpoints, and restart/uncertain-result reconciliation.

### Modified Capabilities

- `workspace`: Bind sync to repository identity across workspace/tab changes; coordinate file-tree mutations and preserve foreign/untitled buffers.
- `reliable-file-persistence`: Add a repository write barrier covering in-flight saves, external reloads, and Git worktree updates without weakening atomic-save/recovery guarantees.
- `document-resources`: Validate attachment participation and portability, coordinate resource writes, and invalidate images changed by Git without reparsing unchanged Markdown.
- `chrome-platform`: Add workspace-specific sync entry/status, sidebar, optional diff/history, and secondary commands while retaining active-document branch semantics.
- `theme-preferences`: Add Git executable and opt-in remote-check settings; persist repository policies independently of recent-workspace eviction.
- `ui-i18n`: Localize setup, sync, errors, credentials, conflict resolution, and recovery surfaces in all supported languages.
- `crate-architecture`: Introduce a GUI-free `markion-git-sync` member and root-app adapters with cached status and no Git work on the typing/render path.

## Impact

- New `crates/git-sync/` member (package `markion-git-sync`) and root-app sync controller/UI modules. System Git is an optional runtime prerequisite for this feature, not for opening or editing documents; no bundled Git distribution is introduced.
- Integrations: `src/app/status_bar.rs`, `application.rs`, `documents.rs`, `workspace.rs`, resource/export write entry points, `src/lib.rs` checked save/reload boundaries, `src/model.rs`, `src/storage/`, `src/paths.rs`, menus, shortcuts, and `src/i18n.rs`.
- Local persistence: additive global settings, a versioned repository-policy file separate from `session.toml`, and operation/draft storage outside repositories. Git retains ownership of remotes, upstream, and repository author configuration; credential helpers own saved secrets.
- Existing completed but unarchived `add-recent-workspace-switcher` and `configure-silent-save-in-preferences` behavior is the integration baseline. This change does not revert retained dirty tabs or recovery-only autosave, and does not directly edit stable specs.
- Preserve per-document-version `Arc` Markdown caches, syntax-highlight memoization, cached text handles, bounded tree rendering, and GUI-free member crates. Git status changes alone never increment document versions.
- Validation: real bare-remote/two-clone integration tests, save/network interruption fault tests, three-platform setup/credential/conflict smoke checks, and `cargo test --workspace` before delivery. No external repository is modified as part of creating this proposal.
