## Context

The confirmed user priority is personal notes across computers with one-click sync. Git is the transport and durable local history; routine use must not require staging knowledge, a commit-message form, or repeated approval of unchanged policy. Collaboration-grade conflict safety is still necessary because one person can edit the same note on two devices.

Current integration points:

| Existing area | Consequence |
| --- | --- |
| `src/app/status_bar.rs` reads `.git/HEAD`, including gitdir indirection, every two seconds | Branch indication exists, but it is active-document context, not authority to synchronize that repository. |
| `WorkspaceSnapshot` and `src/storage/session.rs` store per-root paths | Repository policy must outlive the bounded recent-workspace list and must not contain dirty text. |
| `add-recent-workspace-switcher` retains foreign dirty tabs | Select buffers by resolved repository membership, never by the current tab or all visible tabs. |
| `run_due_autosave` dispatches background writes and `check_external_changes` dispatches reads | Merely disabling timers does not prevent an already-running write or stale reload from racing checkout. |
| `reliable-file-persistence` uses disk identities, atomic replacement, and recovery | Keep these boundaries and add repository coordination around them. |
| `configure-silent-save-in-preferences` permits recovery-only autosave | Explicit Sync Now saves eligible named documents even with silent save off; background fetch never does. |
| The document tree filters file types and directories | Git status needs an independent complete inventory. |
| Managed images have document-relative URLs | Include their bytes in the approved policy and invalidate changed images independently of Markdown source versions. |

Some completed changes are not archived and stable specs still describe older session/autosave behavior. Apply against the current code and those changes, retain their behavior, and reconcile archive ordering rather than restoring obsolete contracts. The added requirements here do not replace their complete requirement blocks.

## Goals / Non-Goals

**Goals:**

- A configured notes workspace synchronizes through one user action with automatic local commits and messages.
- Clone/connect/initialize onboarding works for ordinary repositories over HTTPS and SSH.
- Distinguish memory, disk, local history, and last-confirmed remote state.
- Preserve unsaved content, existing staging, attachments, local commits, and conflict drafts across errors and interruption.
- Keep all process/network/scanning work off the GPUI render and typing paths.
- Deliver the full manual conflict/recovery loop before exposing the feature as ready.

**Non-Goals:**

- Background commit, merge, or push; automatic last-writer-wins conflict choices.
- General branch management, interactive rebase, stash UI, force pushes, history rewriting, whole-repository restore, host-specific repository creation APIs.
- Bundling Git; writing linked worktrees, submodules, LFS-managed repositories, bare/sparse/partial/shallow repositories. Detect these and explain limitations without changing them.
- Synchronizing credentials, application settings, layout, session records, or recovery directories between computers.

## Decisions

### 1. Personal-notes policy approved once

Onboarding binds one local worktree, one named local branch, and one remote branch. It shows the complete repository inventory and history-transfer implications, then records:

- Repository-relative roots approved for changes to already-tracked files, including deletions. Existing unrelated tracked files can be excluded; their changes remain visible and can block integration.
- Roots and file classes approved for new files. The recommended new-file set uses existing Markdown/text extensions and supported image extensions. New dotfiles, hidden paths, symlinks, nested repositories, and arbitrary binaries are attention items unless explicitly accepted through a supported path-specific action; an extension match alone does not authorize them.
- An optional editable default message template. The built-in message is deterministic English (`Sync notes: N files`) with Git's ordinary author/time metadata; device name inclusion is opt-in. Interface language changes do not churn the template.

A dedicated notes repository is the recommended setup. Initial candidates follow the selected notes roots and managed resources, not the entire directory indiscriminately. User-approved tracked scope is independent from new-file eligibility: do not silently stop tracking a renamed or unsupported-extension file. A rename crossing scope boundaries requires review; accepted movement is recorded atomically as both path changes. No attachment garbage collection or automatic empty-directory placeholder creation is introduced.

After setup, Sync Now computes the current eligible change set and commits it without another dialog. Unexpected untracked files and tracked changes outside policy produce an attention list before automatic commit; the user can explicitly include supported paths once, adjust policy, or leave them excluded. Git-ignored untracked files do not require attention unless referenced by a participating document. Existing tracked files remain tracked even when an ignore rule matches. Optional explicit review pins content identities; subsequent edits invalidate that reviewed plan. In the normal one-click path, content is captured after the write barrier and is authorized by the saved policy.

File scope controls the newly created commit only. Fetch/merge affect the whole worktree and push transfers the selected branch's required history, including previously committed files outside the notes scope. Setup and outgoing-history details state this plainly. Existing history is not scanned or rewritten to promise removal of secrets.

Alternative: ask for file selection and a message on every click. Rejected because that contradicts the confirmed one-click priority. Alternative: stage everything under the file-tree root. Rejected because the tree is incomplete and the root may be a repository subdirectory.

### 2. Repository identity is separate from workspace and document context

`RepositoryIdentity` contains canonical worktree root, per-worktree Git directory, and common Git directory, resolved with Git discovery. Resolve path aliases without using lowercased display strings as identity. Bind repository-relative paths and retain native path representations; reject unsupported/unrepresentable paths explicitly rather than round-tripping through lossy strings. Validate lexical and canonical containment for writes and never traverse symlink/junction escapes.

`SyncTarget` contains the named local branch, remote name, resolved fetch and push endpoints, and full destination ref. Multiple push endpoints, mirror configuration, or unsupported ref mappings block one-click setup. A user-visible change of branch, endpoint, upstream, repository identity, or scope invalidates the binding and requires reconnection/review. Do not automatically replace an existing remote configuration.

The workspace header's sync action uses its connected repository even if the active tab belongs elsewhere. The existing document branch indicator keeps active-document semantics. Opening another repository's details is an explicit context switch. An operation captures its identity/target/epoch and never retargets itself when the user changes tabs or folders.

Controllers are keyed by repository identity. Serialize their operations and use a common-directory coordination key as a conservative shared-ref boundary. Persisted policy keys include canonical identity; if a folder is moved/replaced, offer reconnection rather than auto-adopting a different repository at the old path. Remote URL alone is not local identity.

Alternative: attach sync state to `EditorTab` or only to recent-workspace entries. Rejected because tabs can be foreign and recent entries are evicted.

### 3. Three onboarding routes, one binding model

| Route | Behavior |
| --- | --- |
| Clone remote | Validate HTTPS/SSH URL, authenticate, inspect advertised branches/default HEAD, choose an absent or empty destination, clone into an app-owned adjacent temporary directory, and activate after success. Refuse destination replacement if another process populated it. Cancellation cleans only positively identified operation-owned paths. |
| Connect existing repository | Discover ordinary worktree; show current branch/remotes/upstream, capabilities, staging and in-progress operations; select one target and approve notes policy. |
| Initialize notes folder | Review candidates and identity; initialize without replacing user files; create local first commit; attach an empty remote and explicitly establish initial branch/upstream. If the remote already has history, offer clone-and-copy migration, not unrelated-history merge. |

Use the remote's advertised default branch where available; if absent/ambiguous or the remote is empty, ask for the initial branch during setup (suggest `main`, never treat it as discovered). Connecting to a nonempty remote with a missing selected branch requires explicit publish-new-branch intent. A previously existing target that disappears does not get silently recreated.

First sync has a setup review of candidate files and outgoing history. Subsequent ordinary sync has no confirmation. Cancelling setup preserves the current workspace; local initialization already completed before a network failure remains visible and reconnectable instead of being destructively rolled back.

The application presents these routes as a compact quick-start sheet rather than a Git configuration form. If the open folder is already an ordinary repository, **Use This Folder** is primary. Otherwise **Clone Notes Repository** asks only for the remote URL and destination (derived from the repository name), while **Start Syncing This Folder** asks for a remote URL only when publishing is wanted. The suggested initial branch is `main`; discovered upstream/default branches replace the suggestion. Repository scope, endpoints, author identity, and the whole-history explanation remain visible in a review/advanced section before writes are authorized.

Clone and initialize run in background tasks and reuse `OnboardingService`; workspace activation happens only after the destination is complete. Initialization against an empty remote persists a resumable first-publication state so a network or authentication failure can be retried without repeating initialization or losing the local commit. The ordinary configured state collapses setup and keeps **Sync Now** as the primary action.

### 4. System Git behind a GUI-free boundary

Add `crates/git-sync` (package `markion-git-sync`) for repository discovery, state/plan models, process adaptation, execution policy, and recovery reconciliation. Put GPUI scheduling, dialogs, document adapters, and image conversion in root-app modules. The new crate never imports `gpui` or owns live editor entities.

System Git is the first backend to retain established credential helpers, SSH configuration, merge behavior, hooks, and signing. No parallel libgit2 or gix backend is added speculatively. A small operation-oriented boundary permits test doubles without mirroring every Git command.

| Concern | Contract |
| --- | --- |
| Discovery | Explicit configured executable, then normal platform discovery; record version/capabilities and show actionable failure. Initial target floor is Git 2.39 plus actual probes for used features; support is not a security endorsement of an old installation. |
| Arguments | Native argument vectors, validated refs/remotes, explicit working directory, literal pathspec handling and `--` where supported. No shell interpolation or user-provided command templates. |
| Status | `status --porcelain=v2 -z --branch`, complete repository scope, native/NUL-delimited paths, both rename paths. Status reads avoid optional index locks where supported. |
| Diff/history | Disable external diff/textconv; request bounded pages and byte limits. Show source differences for text and metadata/preview for binary. Read-only inspection never launches repository-defined preview commands. |
| Environment | Sanitize inherited Git directory/index/config-injection variables; preserve explicitly supported user configuration and credential/SSH environment. Do not auto-add `safe.directory` or bypass trust failures. |
| Progress | Drain stdout/stderr concurrently with bounds; structured outcomes plus sanitized diagnostics. No terminal prompt may invisibly hang a task. |
| Processes | Hidden Windows child windows; cancellation covers owned child processes and helpers, with stage-specific reconciliation before another write is allowed. |
| Hooks/signing | Respect existing hooks, filters and signing during explicit writes; do not silently bypass failures. Explain configured helper/program execution during connection. If a hook changes the planned index or worktree, revalidate and stop unattended continuation. |

HTTPS and SSH are the interactive onboarding protocols. Local filesystem remotes are allowed only for test fixtures, not inferred from arbitrary URL text in the user form. Unsupported URL schemes/remote-helper transports are rejected.

Explicit setup and synchronization operations enable system Git's foreground credential path. Existing secure helpers, OS credential UI, SSH agents, SSH configuration, and host verification run in their normal foreground context; Markion does not copy the resulting secret into policy, session, operation logs, command arguments, or remote URLs. Authentication-required outcomes keep local work and expose **Retry** after the user completes the system prompt or credential setup. Background checks use a separate noninteractive capability gate and never inherit the foreground setting.

Initial bounded defaults: 15 seconds for ordinary metadata commands, 60 seconds network inactivity with a visible extension/retry action, and a visible long-operation state after 10 seconds for local mutations. Never assume a killed mutation rolled back. Text diff display limit: 2 MiB per file and 10 MiB per view; larger files show metadata and an explicit external-open option. History pages contain 50 commits. These are app presentation/runtime limits, not remote hosting limits.

### 5. Independent status dimensions and a small everyday surface

| Dimension | Representative values |
| --- | --- |
| Connection | Not configured, available, authentication needed, offline, unsupported |
| Documents | Unsaved count, external-disk conflict count, omitted untitled count |
| Worktree | Modified/new/deleted/renamed paths, external staging, conflicts |
| History relation | Unknown, equal, ahead, behind, diverged, unrelated, rewritten/deleted target |
| Operation | Idle, preparing, saving, committing, fetching, integrating, resolving, pushing, verifying, needs attention |

Counts for ahead/behind are relative to a known remote snapshot and display its checked time. No upstream means unknown, not zero. A successful operation can coexist with new local edits. Offline state retains the last confirmation timestamp.

```text
My Notes                                      [Sync Now]
Saved locally - 3 changes
Last synchronized: today 14:32                 [View Changes]

Sync sidebar details:
Repository / remote / branch / last check
Complete changes | outgoing commits | recent history
Activity and actionable errors
More: Commit Locally / Check Remote / Pull Updates / Push Commits
```

The sidebar is optional during routine sync. Existing workspace-header click/drag behavior stays intact by using a distinct sync control. File-tree decorations only annotate already-visible rows; omitted files still appear in the full Git list. Commands use existing action registration and customizable shortcuts without assigning conflicting defaults. Compact widths collapse details rather than overlapping the existing status row. Status is conveyed by text/icons as well as color.

Recent history is read-only with commit/file diff and Save a Copy from a historical blob; restoring an entire repository or rewriting a current branch is deferred. Any copy saved into the repository uses the same write coordination as other file writes.

### 6. Operation semantics and checkpoints

| Action | Saves / creates local commit | Fetches / integrates | Pushes |
| --- | --- | --- | --- |
| Sync Now | Yes, eligible named buffers and policy-approved changes | Yes / yes if safe | Yes |
| Commit Locally | Yes, same scope rules | No | No |
| Check Remote | No | Yes / no | No |
| Pull Updates | No implicit buffer save or snapshot commit | Yes / yes if clean; divergence can create a merge commit | No |
| Push Commits | No | Only reads needed to validate target/result; no integration | Fixed existing commit |

Sync Now performs:

1. Acquire the per-repository operation slot. Capture binding and epoch. Check repository capability/state, policy exceptions, disk conflicts, existing index changes, identity/signing readiness and last operation journal. External staging or merge/rebase/cherry-pick/revert state blocks new one-click writes.
2. Acquire the repository write barrier, drain existing writes, resolve eligible named buffers by actual repository identity, preserve recovery, and save through checked atomic persistence. Ignore foreign buffers; report untitled/out-of-policy buffers as excluded. If any eligible save fails, stop before staging or commit; already-successful saves remain valid local changes.
3. Capture exact disk identities and eligible path set. Record journal intent, stage only those paths using literal semantics, verify the index tree, then commit with generated/default message if different from HEAD. Verify resulting HEAD/index/worktree. An existing partial staging arrangement is never adopted. If a failed commit leaves app-staged paths, record ownership and offer retry or unstage only after verifying unchanged index identity; never reset unknown staging. On restart a matching operation-owned staging state is recoverable, not misclassified as arbitrary external staging.
4. Record the resulting local commit and release the worktree barrier. Fetch the bound remote without recursively updating submodules. On network failure stop with a preserved outgoing local commit. No empty commits are created when there is no content change.
5. Compare named branch/HEAD, fetched target, and prior remote observation. Detect force-rewritten or deleted remote targets before ordinary ancestry classification. A first connection with unusual histories is reviewed during setup. Use fixed fetched OIDs for integration, not an ambient `FETCH_HEAD` overwritten by another operation.
6. Equal: no integration. Local ahead: prepare push. Remote ahead: fast-forward. Diverged with a merge base: ordinary merge, preserving both histories. Unrelated/rewritten/deleted/ambiguous target: stop. Never automatic rebase, stash, force, tag push, mirror, or branch deletion.
7. Before fast-forward/merge, reacquire the write barrier and revalidate captured HEAD/binding and clean tracked/index/buffer state across the entire repository. New edits made during fetch pause integration instead of being auto-committed repeatedly. Untracked/ignored files that would be overwritten, path aliases/case collisions, escaping paths, unsupported symlink transitions, or incomplete materialization block checkout. Validate incoming changed paths before mutation. Persist pre-integration OIDs, relevant file identities, and operation-owned recovery references before beginning.
8. During conflict, transition to the durable conflict session below. After clean integration, refresh documents and images before releasing the barrier. Hold only repository coordination, never a GPUI thread lock during Git execution.
9. Revalidate the configured target and expected branch; push a fixed local commit OID to the exact full remote branch ref with non-force semantics and without implicit tags/multiple destinations. New uncommitted edits do not change that OID and remain pending. A changed branch/HEAD requires attention rather than silently expanding this operation.
10. Parse the per-ref result and fetch/inspect the destination when confirmation is missing. A remote tip containing the intended commit confirms its delivery; equality with current local HEAD is not required to establish delivery. If remote tip also has newer commits, show incoming changes, not global equality. An ordinary rejection permits at most one re-fetch/re-integrate/re-push attempt under the same clean-state rules; do not create a second content snapshot commit.
11. Recompute local state, persist last confirmation, and report exact progress (for example, synchronized captured changes with new edits remaining). Confirmation failure yields an uncertain state, never a false success or an automatic destructive rollback.

Alternative: `git pull && git push` with ambient defaults. Rejected because it obscures save ordering, config-dependent rebases, target drift, and partial success. Alternative: automatic stash on dirty worktrees. Deferred because stash restoration adds another conflict lifecycle and complicates abort guarantees.

### 7. A write barrier that covers writes already in flight

The root app owns a repository-aware coordinator used by explicit/autosaves, Save As (target repository), rename/move/delete, image import/replacement, exports into the repository, history-copy writes, quit-save, and Git worktree mutations. Normal writes take compatible admission tokens; Git worktree mutation takes exclusive admission and drains previously admitted writers. Discovery/registration cannot be bypassed by creating a new path under a locked root. Cross-root moves acquire involved gates in stable order.

```text
UI action -> repository controller -> request exclusive mutation
                                    -> stop admitting new app writes
                                    -> drain pending disk writes + apply completions
                                    -> invalidate outstanding disk-read epochs
                                    -> save/check buffers + freeze affected editing
                                    -> journal + Git worktree mutation (background)
                                    -> reconcile disk identities and changed paths
                                    -> checked document reload / image invalidation
                                    -> release gate + resume editing and autosave
```

Do not merely bump `autosave_generation`: that stops future timers or stale UI application, not an actual write already running. Existing completions must settle before Git establishes its disk baseline. Outstanding external-file reads carry a repository epoch in addition to document instance/version; pre-mutation results cannot reload after Git updates disk. While a conflict session owns paths, ordinary autosave cannot write them; draft persistence is separate and continues.

Network fetch/push permits ordinary editing/writes; local snapshot capture and worktree replacement briefly freeze affected editing. If hooks or large checkouts take time, show progress and a safe cancellation request. Cancel waits for a safe boundary or terminates the owned process tree then reconciles journal/HEAD/index/worktree before releasing write permission. App exit never starts a competing save; it waits for a local mutation boundary or records resumable state. No long network operation is required to finish before a safe exit.

An app mutex is not a filesystem transaction and cannot lock out external Git/editors. Respect Git lock failures, validate identities before and after mutation, stop on unexpected divergence, retain available preimages/checkpoints, and never delete an unknown lock or run unconditional reset/clean to repair it.

### 8. Reconcile tabs and attachments without cache churn

Only actual source-byte changes go through `apply_external_reload_checked` or its extended equivalent, once per accepted transition. Retain tab identity, avoid rebuilding unrelated caches, and preserve caret/viewport where valid. Existing reload/undo lifecycle rules apply; restoration of pre-sync content is exposed as a recovery/history copy rather than pretending old undo entries still refer to the new disk baseline.

For a confidently identified rename, remap a clean tab. For ambiguous rename treat paths as deletion/addition. A deleted open document remains as a recoverable missing-destination view with ordinary autosave disabled; do not recreate it automatically or clear its retained source. Save a Copy can create a new path explicitly. Dirty tabs are not reloaded by Git.

Invalidate image-tab/preview cache entries for changed resource paths even if referring Markdown bytes are unchanged. Preserve authored relative URLs and metadata. Check newly participating note references against tracked or selected resources, and identify ignored, missing or out-of-repository resources. Offer existing resource organization/copy only on user action. Remote URLs and data URIs retain their current semantics. Newly referenced omitted attachments require a one-time include/fix/continue-with-known-omission choice, not a repeated dialog for a previously acknowledged unchanged reference.

### 9. Conflict sessions are separate from ordinary document disk conflicts

Never enter integration until existing dirty-buffer/disk conflicts have been resolved through their current UI. For a Git merge conflict, record operation ID, base/local/remote OIDs and indexed conflict stages. Ordinary document views of conflict-owned paths become read-only with a link to the resolver; pre-existing views cannot silently autosave merge-marker text.

The resolver shows local, base and remote source plus an editable result. Use personal language (This Computer / Remote Version) with branch and commit metadata in details, since a remote commit may originate from a different device or person. Support per-hunk local/remote/both choices and full-source edits; Markdown preview is optional and is not the conflict source of truth.

| Conflict | Resolution |
| --- | --- |
| Text modify/modify or add/add | Compare stages, edit source, explicitly mark resolved. |
| Delete/modify | Choose deletion or preservation, with source preview. |
| Binary/image | Compare previews/metadata; choose side or save both under validated distinct paths. |
| Rename conflict | Show original and destination paths; simple choices in app, complex graph delegated while preserving state. |
| Unsupported encoding/path structure | Explain limitation, retain conflict, allow external resolution and refresh. |

Draft save does not stage or complete a resolution. Explicit Mark Resolved writes the chosen result through the coordinator and updates the conflict index only after verifying session OIDs and current path identities. Consult index stages, not only marker-string scanning. A legitimate Markdown document can quote conflict markers. Keep-both does not guarantee valid Markdown or correct references and exposes the result for inspection.

Finish Merge is enabled only after all index conflicts are cleared and draft versions match written results. It creates the merge commit, respects signing/hooks, and returns to the sync flow; after interactive resolution provide an explicit Finish and Push action rather than surprising background upload. Abort persists user drafts first, verifies that this app owns the merge state, and uses supported Git abort behavior. Retain the local snapshot commit created before merge. External repository changes block automatic abort; offer recovery copies and external inspection instead. External resolution can be adopted only after comparing the actual merge/HEAD/index state.

### 10. Durable journal and recovery ownership

Use atomic versioned journal and draft files under the app-local data directory, outside every synchronized repository. Write intent before staging, committing, integration and push; write confirmation after inspecting actual outcomes. Store OIDs as opaque strings, target identity, index tree/fingerprint, selected path identities, stage, and timestamps. Do not store credentials or unbounded raw output. Durable local references keep operation baseline commits reachable while recovery is pending; never push those private recovery refs and remove them only after confirmed retirement.

File preimages needed for interrupted checkout/recovery use a bounded per-operation manifest and durable blobs outside the worktree. If required preimages cannot be persisted (space/permissions), do not start the worktree mutation. Recovery is offered as copies if the current path no longer matches the expected post-operation identity; do not promise atomic rollback of an entire repository. Capacity checks happen before mutation, not after a partial copy. Recovery cleanup never removes unresolved drafts or user-created files.

On startup, reconcile journal against real Git state before resuming writes:

| Observed state | Recovery |
| --- | --- |
| Saved files, no commit | Keep ordinary changes and document recovery. |
| Owned staged content after failed/interrupted commit | Verify ownership; retry commit or explicitly unstage that unchanged content. |
| Commit exists but journal missed completion | Recognize expected parent/tree and actual HEAD; avoid duplicate commit. |
| Merge in progress | Restore conflict manager and drafts; do not restore ordinary autosave on owned paths. |
| Integration finished | Verify HEAD/tree and refresh affected tabs. |
| Push outcome unknown | Re-fetch target and check whether it contains the intended commit. |
| External state differs | Needs attention; never reset to journal assumptions. |

Retire successful operation data only after durable confirmation and no unresolved drafts. Keep recent successful summaries bounded (initially 100 records); keep active recovery indefinitely until resolved/discarded explicitly. Logs are not a second authoritative Git database.

### 11. Authentication, settings and background checks

Use configured HTTPS credential helpers first, with a controlled askpass bridge for missing interactive input. Offer session-only credentials or storage through a detected secure helper; if none is available, do not configure plaintext credential storage. SSH uses user configuration, agent and known_hosts with explicit handling for unknown/changed host keys and encrypted keys. Never disable TLS/SSH verification to make sync work. Connection settings expose author name/email separately from remote login, default author changes to repository-local config, and keep signing requirements.

Treat credential input, URL userinfo, helper output, stderr and redirects as sensitive for logging. Do not put tokens in process arguments or remote URLs. Credential helpers can open login UI only for explicit user operations; opt-in background fetch must use noninteractive capability or stop with Authentication Needed. If a helper cannot be reliably made noninteractive, background fetch for that binding is disabled.

Persist:

| Data | Owner/location |
| --- | --- |
| Executable override, global background-check default | Additive `[git]` in current `config.toml` serializer |
| Repository target fingerprint, approved scope/new-file rules, message, per-repo check setting | Versioned `git-sync.toml` in app config directory |
| Remotes/upstream/repository identity settings | Existing Git config, changed only through explicit setup |
| Credentials | Existing secure helper/agent or process-session memory |
| Operation journal, conflict drafts and recovery preimages | App-local Git sync data directory |
| Workspaces/tabs | Existing `session.toml`, no embedded Git policy |

Missing configuration defaults to no connected repository and background checking off. Preferences reset clears global settings, not repository policy, credentials, or pending recovery. Disconnect removes binding/automatic checks but preserves files, `.git`, remotes, local commits and unresolved recovery. A policy corrupted or with an unsupported schema disables writes and reports repair; do not default to a broad include policy.

Opt-in background fetch checks only the currently connected active workspace, at most one request at a time, initially every five minutes while active, with jitter and error backoff capped at thirty minutes. Activation can request a check if stale; it does not bypass backoff continuously. Switch-away stops future scheduled checks; running results remain bound to their repository. No background action saves a buffer, creates a commit, integrates worktree changes, pushes, or repeatedly prompts. Notify only on incoming changes or actionable failures, not unchanged polls.

### 12. Verification and delivery gates

Use local bare remote plus two ordinary clones for topology tests, with isolated Git/credential configuration and deterministic identities. Test HTTPS/SSH separately with controlled fixtures and opt-in manual accounts; CI must not write a user's real remote or use ambient credentials. Fault seams cover process interruption, partial output, authentication failure, rejected push, accepted-but-unacknowledged push, disk-full preimages, and external mutations.

Root-app tests exercise retained foreign/untitled tabs, silent-save off, all admitted writer classes, in-flight autosave drain, stale external reads, quit-save, conflict ownership, deleted documents, and changed-image-only reload. Verify status refresh does not change document versions, undo state or derived cache identity. A 10,000-file synthetic notes repository is a responsiveness fixture: Git work stays asynchronous, no unbounded process queue is created, lists/diffs/history are bounded, and typing tests do not synchronously call Git.

Document Windows/macOS/Linux executable, HTTPS helper, SSH-agent, clone, empty remote, two-device merge, crash recovery and path compatibility smoke results. Run `cargo test -p markion-git-sync`, relevant root regressions and `cargo test --workspace`. Do not mark the feature delivered with the conflict/restore gates pending.

## Risks / Trade-offs

- [System Git is absent or differs across machines] -> Detect version/features, offer executable configuration and actionable setup instructions, retain normal editing.
- [External Git/editor bypasses app coordination] -> Git locks plus identity/epoch checks; preserve checkpoints and stop rather than claim universal transactional exclusion.
- [One-click policy surprises users in mixed repositories] -> Recommend dedicated notes repo, explicit initial scope/history explanation, complete unexpected-change inventory, no silent scope expansion.
- [Remote force rewrite, protected branch, or credential expiry] -> Distinct actionable state; no force-push, bypassed hooks, or false synchronized indicator.
- [Local merge produces conflict or checkout stops partway] -> Require clean baseline, durable preimages/OIDs, draft ownership and tested restart reconciliation before shipping.
- [Line endings, Unicode/case, symlinks, file locks differ across platforms] -> Respect Git attributes/config without silent normalization; validate incoming paths and materialization; unsupported transformations stop before applying where detectable.
- [Filters/LFS/special repositories exceed initial compatibility] -> Explicit detection and disabled write actions; never treat missing content or LFS pointers as synchronized images.
- [Merge/history and recovery storage grow] -> Bounded display/log caches and retirement of confirmed owned recovery; no implicit Git GC/history rewriting or deletion of unresolved drafts.
- [Scope is large despite simple UI] -> Stage implementation internally, but release only once the manual save/commit/merge/push/conflict/recovery path passes. Background writes remain a separate future change.

## Migration Plan

1. Introduce core models/process adapter and headless fixtures, then local settings with safe missing-field defaults. Do not change existing repositories on app launch.
2. Integrate write admission and checked reload epochs across existing writers before enabling Git worktree mutation in UI.
3. Implement onboarding, scope and manual operations, followed by complete conflict and restart recovery.
4. Add compact UI, optional inspection and opt-in background fetch, all localization and user docs; pass automated and platform gates.
5. Archive only after tasks and validation complete. Reconcile the unarchived workspace/autosave baseline; stable specs change only via normal delta sync/archive.

Rollback disables the feature and scheduling without altering user repositories or dropping journals. Old app versions ignore additive global keys and the separate Git files. If rollback occurs during an unresolved merge, retain recovery and explain how to finish/abort with Git; never make downgrade silently write over that state.

## Open Questions

No product decision blocks implementation. The one-click priority, manual-write scope, system-Git backend, ordinary merge policy, and deferred capabilities are fixed for this proposal.

Implementation validation items are explicit tasks: verify feature probes/minimum Git on each platform; prove credential-helper background noninteraction; exercise process-tree cancellation and interrupted-checkout recovery. If any cannot meet this contract, keep the affected operation disabled and revise this change's planning artifacts before widening support.

## References

- [Git status machine formats](https://git-scm.com/docs/git-status)
- [Git merge, pre-merge checks and abort limitations](https://git-scm.com/docs/git-merge)
- [Git push ref selection and non-fast-forward behavior](https://git-scm.com/docs/git-push)
- [Git credential helpers](https://git-scm.com/docs/gitcredentials)
- [Git ignore and already-tracked files](https://git-scm.com/docs/gitignore)
- [Git worktree model](https://git-scm.com/docs/git-worktree)

These primary references were consulted during the preceding exploration; runtime compatibility still requires the platform probes and tests above.
