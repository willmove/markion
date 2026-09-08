## 1. Core boundary and deterministic fixtures

- [x] 1.1 Add the GUI-free `crates/git-sync` member/package `markion-git-sync`, root dependency wiring, and pure repository/target/status/plan/outcome types; verify no GPUI dependency and preserve root-package cargo behavior.
- [x] 1.2 Add isolated bare-remote/two-clone fixtures with deterministic identity and no ambient user Git/credential configuration; cover ordinary, empty, divergent and unsupported-repository fixture creation.
- [x] 1.3 Implement structured Git process invocation with native/literal path arguments, environment sanitization, concurrent bounded output draining, and typed failures; test spaces, Chinese, leading-dash and unusual valid paths without shell execution.
- [x] 1.4 Implement executable/version/capability detection and configurable override; verify missing/unsupported Git leaves normal editing available and record the tested minimum feature probes.
- [x] 1.5 Implement progress, per-stage timeout, owned-process-tree cancellation and hidden Windows process creation; prove mutation cancellation yields reconciliation-required state rather than assumed rollback.

## 2. Repository discovery and status

- [x] 2.1 Resolve canonical worktree, Git directory and common-directory identities, including alias containment checks; detect nested repositories and refuse escaping or unrepresentable write paths.
- [x] 2.2 Implement ordinary/empty repository support and explicit unsupported-shape checks for detached HEAD, linked worktrees, submodules, LFS, bare, sparse, partial and shallow repositories.
- [x] 2.3 Parse complete `status --porcelain=v2 -z --branch` inventories, index/worktree distinctions, conflicts and both rename paths; verify document-tree omissions do not remove Git changes.
- [x] 2.4 Resolve one remote/local-branch/full-target binding with actual fetch/push endpoints; reject multiple push destinations, mirror/ambiguous mappings and detect later target drift.
- [x] 2.5 Implement timestamped ancestry/ahead/behind/unknown state, previous-target rewrite/deletion detection, and externally active merge/rebase/cherry-pick/revert/lock detection with real Git fixtures.
- [x] 2.6 Implement bounded source diff and recent/outgoing history pages with external diff/textconv disabled, plus binary/image metadata and historical-blob reads.

## 3. Policy and local persistence

- [x] 3.1 Add additive `[git]` executable/background-check preferences and atomically persisted versioned `git-sync.toml` repository policies outside session storage; test safe defaults and corrupt/unsupported schema handling.
- [x] 3.2 Implement approved tracked roots and approved new-note/resource roots/classes, Git-ignore behavior, unexpected-file exceptions and cross-scope rename decisions; test that hidden or unrelated files never enter by tree-filter inference.
- [x] 3.3 Implement automatic/default message generation and optional edited messages/review plans; test that ordinary policy-approved sync needs no per-run form and reviewed content drift invalidates approval.
- [x] 3.4 Implement policy identity validation, reconnect, disconnect, and reset/eviction behavior; verify recent-workspace eviction or Preferences reset does not remove repository policy, credentials or recovery.
- [x] 3.5 Add atomic operation intent/confirmation records, durable recovery references, preimage manifests/blobs and conflict draft storage; test persistence failure prevents starting protected mutations.
- [x] 3.6 Implement bounded successful-summary retirement and preservation of unresolved operation/draft data; test ownership checks prevent cleanup of unknown or user-owned files.

## 4. Authentication and onboarding

- [x] 4.1 Add HTTPS helper integration and controlled askpass interaction with session-only/secure-helper storage choices; verify no plaintext helper configuration or secret-bearing command arguments/URLs/logs.
- [x] 4.2 Add SSH-agent/config/known-host handling with explicit unknown/changed-host and encrypted-key outcomes; verify cancellation and verification failures do not bypass security checks.
- [x] 4.3 Add separate repository-local author configuration and signing/hook error propagation; test failed signing/hooks never become silent unsigned/bypassed success.
- [x] 4.4 Implement validated HTTPS/SSH clone and remote-default/explicit-branch selection, including empty remotes; verify operation-owned temporary directories and destination-race/cancellation cleanup.
- [x] 4.5 Implement connect-existing and initialize-notes flows with initial inventory/history/scope review; test no silent remote replacement and no unrelated-history integration.
- [x] 4.6 Implement explicit first publication/initial upstream binding and safe resume after setup network failure; distinguish an intentionally new branch from a previously existing deleted target.
- [x] 4.7 Wire `OnboardingService` into a notes-oriented application quick start for use-current-folder, clone, initialize and resumable first publication; derive safe defaults, switch workspaces only after clone success, and test the ordinary first-use paths.
- [x] 4.8 Enable secure system foreground credentials for explicit clone/sync operations with actionable authentication retry, while keeping background checks noninteractive and secrets out of persisted state and diagnostics; add boundary tests.

## 5. Application write admission and document lifecycle

- [x] 5.1 Add root-app repository operation registry and write-admission tokens keyed by worktree/common-directory identity; test serialization and duplicate-click coalescing.
- [x] 5.2 Route manual save and autosave through admission, drain already-running writes and apply their completions before exclusive Git mutation; add the old-autosave-after-pull regression test.
- [x] 5.3 Route Save As, quit-save, file-tree create/rename/move/delete and cross-root moves through destination-aware admission; test no writer bypass based on the source workspace.
- [x] 5.4 Route resource import/replacement, exports into a repository and historical-copy writes through the same admission boundary; verify failed/deferred imports do not insert dangling references.
- [x] 5.5 Add repository epochs to pending external reads and checked reload application; test old disk results cannot replace post-Git source/identity.
- [x] 5.6 Add integration buffer capture/freeze/resume and membership filtering across retained foreign/untitled tabs; test explicit Sync Now honors checked saves when silent save is disabled without changing preferences.
- [x] 5.7 Implement checked post-Git text reconciliation and clean rename mapping; test source-version/cache changes only for affected content and retain existing lifecycle/undo behavior.
- [x] 5.8 Implement recoverable missing-destination state for Git-deleted open files with autosave recreation blocked; test explicit Save a Copy and ambiguous rename handling.
- [x] 5.9 Implement safe local-mutation cancellation/quit transitions and admission restoration after reconciliation; test in-flight completion and pending recovery survive without competing quit-save writes.

## 6. Local snapshots and remote synchronization

- [x] 6.1 Implement preflight gates for unsupported capabilities, target drift, foreign staging, disk conflicts, policy exceptions and active operations; verify refusal preserves repository state.
- [x] 6.2 Implement policy-authorized save/capture/stage/commit with exact path/index checks and no empty commits; test approved note additions, modifications, deletions, attachments and optional reviewed-plan staleness.
- [x] 6.3 Implement app-owned staging recovery after commit failure/interruption, with verified retry/unstage; test external index changes are never reset or absorbed.
- [x] 6.4 Implement fetch without document/worktree mutation and retain local snapshots on offline failure; test fixed fetched-OID capture independent of ambient `FETCH_HEAD` changes.
- [x] 6.5 Implement equal/ahead/behind/diverged dispatch, clean full-repository preconditions and safe fast-forward/merge; test no implicit rebase/stash/reset/clean and no repeated snapshot commit after fetch-time edits.
- [x] 6.6 Validate incoming path materialization, untracked/ignored collisions, case aliases and unsupported symlink transitions before integration; test rejection preserves local paths and uses required preimages/checkpoints.
- [x] 6.7 Implement exact-OID, single-full-ref non-force push with target/branch revalidation and no implicit tags/mirror/all-branch transfer; test new working edits do not expand the pushed snapshot.
- [x] 6.8 Implement one bounded non-fast-forward retry and remote-history verification for lost push responses; test accepted-but-unacknowledged delivery, continuing remote advancement and unresolved uncertainty.
- [x] 6.9 Compose Sync Now and secondary Commit Locally/Check Remote/Pull Updates/Push Commits contracts; verify distinct effects, partial-success messages and new-local-edits-after-delivery status.

## 7. Conflict resolution and restart recovery

- [x] 7.1 Create conflict sessions from actual index stages and captured operation/commit identity; make ordinary tabs for owned paths read-only and persist separate resolution drafts.
- [x] 7.2 Implement local/base/remote source comparison, hunk choices, manual result editing, draft saving and ownership-checked Mark Resolved; test literal marker text is not the resolution authority.
- [x] 7.3 Implement delete/modify and add/add choices with explicit source/path presentation; test neither timestamps nor file existence silently choose a winner.
- [x] 7.4 Implement image/binary side choice and validated keep-both paths; test simple rename decisions and preserve unsupported conflicts with external-tool guidance.
- [x] 7.5 Implement Finish Merge and explicit Finish and Push with current draft/index verification and signing/hook preservation; test incomplete or stale resolutions block completion.
- [x] 7.6 Implement ownership-checked Abort with durable drafts and pre-merge local snapshot preservation; test external HEAD/index changes prevent unsafe automatic abort.
- [x] 7.7 Implement startup reconciliation of owned staging, commit-completed-before-journal, conflict sessions and completed integration; prove retries do not duplicate content commits.
- [x] 7.8 Implement interrupted checkout and unknown push-result recovery against real paths/refs/index; test independently modified paths yield safe copies/attention rather than automatic reset/clean.
- [x] 7.9 Implement adoption of externally completed resolution and disconnected pending-recovery access with continued path guards; verify unresolved data survives restart and disconnect.

## 8. Notes and attachment integrity

- [x] 8.1 Implement participating-note reference checks against tracked/selected resources, with ignored/missing/out-of-repository classification; test ordinary approved attachments join the one-click commit.
- [x] 8.2 Connect explicit include/organize/repair/known-omission choices to existing resource workflows and persist scoped acknowledgment identities; test changed references invalidate acknowledgment and no silent URL rewrite/download occurs.
- [x] 8.3 Invalidate changed resource preview/image-tab caches independently from Markdown source; test image-only pull refreshes displayed bytes while retaining unchanged Markdown versions and caches.

## 9. Everyday UI and command integration

- [x] 9.1 Add distinct workspace Sync Now control and compact layered/timestamped status without disrupting root switch/drop gestures or existing active-document branch semantics; test foreign-tab context.
- [x] 9.2 Build Sync setup/details sidebar for repository/branch/scope/identity, complete changes, operation progress and actionable exceptions; verify normal configured sync bypasses repeated commit forms.
- [x] 9.3 Add optional bounded diff/outgoing/recent-history inspection and guarded Save a Copy for historical content, including binary/oversized fallbacks.
- [x] 9.4 Connect conflict/recovery UI with keyboard-accessible source choices, draft/result actions and explicit Finish and Push/Abort; test ordinary editor focus cannot overwrite owned paths.
- [x] 9.5 Register secondary actions and customizable shortcuts without default conflicts, add existing-row Git decorations, and verify compact layouts preserve document metrics/feedback and non-color status cues.
- [x] 9.6 Add Git executable/global check preferences and repository policy controls, reconnect/disconnect flows and clear offline/uncertain-result feedback.
- [x] 9.7 Consolidate global Git settings into the single Git Sync section at the end of General preferences and add a regression test preventing duplicate placement.
- [x] 9.8 Match the Git executable summary and detection result typography to other General preference text and cover both lines with a style regression assertion.

## 10. Optional background fetch

- [x] 10.1 Add default-off active-workspace-only remote-check scheduling with one in-flight request, stale-on-activation checks, five-minute nominal interval and bounded backoff/jitter; test switch-away stops future checks.
- [x] 10.2 Verify helper noninteractive behavior per supported platform and gate background checking when it cannot be guaranteed; test missing credentials pause without recurring login UI.
- [x] 10.3 Connect quiet unchanged results and incoming/actionable-error notifications; prove background checks never save, commit, integrate, push or touch inactive repository buffers.

## 11. Localization and end-user guidance

- [x] 11.1 Add all setup, scope, credentials, sync, conflict, recovery, preferences and capability-limit messages through `src/i18n.rs` for every supported language; run localization completeness checks.
- [x] 11.2 Document personal-notes setup, one-click save/commit/push distinctions, empty/nonempty remote onboarding, HTTPS/SSH prerequisites, whole-history implications and attachment policy in user documentation.
- [x] 11.3 Document offline/uncertain delivery, collision/conflict/restart recovery, unsupported repositories, background-fetch-only scope and safe downgrade/disconnect behavior; update shortcut documentation for actual registered actions.

## 12. Release-readiness verification

- [x] 12.1 Run headless core and two-clone topology suites covering alternating-device sync, nonconflicting/conflicting concurrent edits, binary/delete/rename cases, target rewrite/deletion, offline and one-retry push races.
- [x] 12.2 Run root integration regressions for all writer classes, autosave drain, stale reload epochs, silent-save off, foreign/untitled tabs, deletion, image-only update, quit/cancel and cache invariants.
- [x] 12.3 Run fault-injection cases for disk-full preimages, commit/checkout interruption, external index/path mutations, unknown locks, authentication failures and accepted-but-unacknowledged push; record recovery outcomes.
- [x] 12.4 Exercise a 10,000-file repository and bounded large diff/history cases; verify asynchronous operation, bounded queues/UI lists and no synchronous Git on the typing/render path.
- [ ] 12.5 Record Windows/macOS/Linux smoke evidence for Git discovery, HTTPS helpers, SSH agents, clone/empty remote, conflicts/restart, Unicode/case/line-ending behavior and hidden process behavior where applicable.
- [x] 12.6 Run `cargo test -p markion-git-sync`, `cargo test --workspace`, and `openspec validate add-git-workspace-sync --strict`; reconcile current workspace/autosave changes before marking implementation complete, and leave any unverified gate unchecked.


## Implementation evidence — 2026-09-08

See [verification.md](verification.md) for the implemented everyday sync flow, command results, recovery matrix, and Windows smoke checks. The only remaining release gate is 12.5: macOS and Linux native credential/agent/application smoke evidence cannot be produced from this Windows host, so it remains explicitly unchecked.
