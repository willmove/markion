# Git workspace sync verification

Date: 2026-09-08. Host: Windows. Starting checkout: `76e888a`.

> Status: The original Git-oriented UI evidence below is retained for history. The ordinary-user Backup and Sync revision has current automated evidence below. The project maintainer completed and confirmed the outstanding native smoke and Windows visual/keyboard/user-flow validation on 2026-09-11, so tasks 12.5 and 13.8 are complete.

## Ordinary-user Backup and Sync revision — 2026-09-10

- A table-driven core projection covers off, running, local-pending, incoming, combined-pending, confirmed synchronized, offline, authentication, conflict/recovery, uncertain delivery, attention and remote-unknown states. Critical-state precedence is deterministic, and local-only or stale facts cannot produce the synchronized state.
- The persistent Sync sidebar selection and duplicate Files-panel controls are gone. Legacy `sidebar_tab = "sync"` preferences migrate to Files while repository policy and recovery storage remain untouched. The status bar renders one workspace Backup and Sync entry with one contextual action.
- Source and GPUI regressions cover the transient 640-pixel-bounded Backup and Sync center, narrow-width containment, localized non-color state text, visible focus styling, Tab traversal, Enter/Space activation, cancellation, and preservation of a requested file-history path.
- In-window and native menus expose the ordinary Backup and Sync actions; Commit Locally, Check Remote, Pull Updates and Push Commits are grouped under Advanced Git Tools. Version History is available from File, file-tree and tab context menus, while the existing guarded Restore in editor and Save a Copy tests remain green.
- Onboarding chooses one contextual **use this folder**, **enable Backup and Sync**, or **get notes from a sync address** route, hides alternatives and technical fields by default, labels the remote as a Sync address, rejects an empty required address, and escalates nested, mixed, or unsupported repositories to Advanced Repository Setup. Its bounded tracked-path inventory runs off the UI thread; core integration and application routing tests prove unrelated tracked content cannot receive ordinary authorization. Nested setup shows both the notes-folder and repository roots beside the whole-history consequence. Existing service tests cover cancellation, destination defaults, initialization, first publication and safe resume.
- Conflict work renders as a focused surface with This Computer, Synced and Combined version language. Technical object/index metadata stays behind Advanced Git details. Core two-clone tests cover text, delete/modify, add/add, binary Keep Both, restart recovery, abort ownership checks, and finish/continue synchronization.
- Global Preferences and per-workspace Connection settings distinguish their background-check options and show the same explicit promise that checks never upload notes or apply changes. Executable and repository-policy controls are behind Advanced Git settings. All seven locale entries pass non-empty and placeholder-parity checks, and the user and shortcut guides now describe the ordinary flow.

### Current executed checks

| Check | Result |
| --- | --- |
| `cargo check -p markion` | Passed |
| `cargo test -p markion --lib --no-fail-fast` | Passed: 560 tests, 1 ignored |
| `cargo test -p markion-git-sync --no-fail-fast` | Passed: 53 unit, 3 fixture and 18 integration tests |
| `cargo test --workspace --no-fail-fast` | Passed, including 570 application tests (2 ignored), revised root UI/state, every workspace member and doc tests |
| `cargo fmt --all -- --check` | Passed |
| `git diff --check` | Passed |
| `openspec validate add-git-workspace-sync --strict` | Passed |

### Superseded Windows live-UI environment limitation

The Computer Use runtime was initialized according to its bundled instructions, but two attempts to start its app server failed with `Access denied (os error 5)` before any application window could be selected. No UI input was sent and no personal settings, credentials or remotes were touched. The automated Windows GPUI/source evidence above is current, and the earlier disposable-repository Windows smoke below still validates the synchronization engine, Unicode content and hidden child processes. This limitation was subsequently closed by the project maintainer's confirmed real-device validation on 2026-09-11, covering the Windows visual/keyboard/user-flow walkthrough and the native platform smoke matrix; tasks 12.5 and 13.8 are therefore checked.

## Everyday notes flow

- **Repository → Set Up Git Sync** and an unconfigured **Sync Now** open the same quick start. It selects **Use this notes folder** for an existing repository, **Start syncing this folder** when the current folder already contains notes, and **Clone a notes repository** for an empty non-repository folder.
- Clone needs only an HTTPS or SSH repository address. Its destination is derived from the repository name, and branch/author overrides stay under Advanced setup. The current workspace is retained until cloning succeeds.
- Initialize creates `main` by default, attaches `origin` without replacing an existing remote, stages only reviewed Markdown/text/image paths, creates the first version, and publishes its upstream. Hidden files, arbitrary binaries, external staging, dirty saved buffers and unsupported repository shapes stop the flow.
- Empty clones open successfully without inventing a commit. After the first note is saved, Sync Now resumes the first publication. Failed authentication or publication retains the repository and local commit and exposes Retry.
- Explicit setup and sync enable the system foreground credential path while keeping credentials out of policy, journals, remote URLs, command diagnostics and application state. Background checking uses a separate noninteractive gate.
- All quick-start labels and validation guidance cover English, Simplified/Traditional Chinese, Japanese, French, German and Spanish.
- General preferences render one Git Sync section at the end. The section owns executable selection/detection and background checking, with no duplicate Git controls near the top of the page. Its executable summary and detection result use the same 12px informational text size as the other General settings.

## Synchronization and recovery matrix

- The headless two-clone suite covers alternating-device changes, clean divergent integration, text conflicts, delete/modify, add/add, binary Keep Both, clean rename propagation, rename/rename external-tool fallback, rewritten/deleted targets, offline fetch, one bounded non-fast-forward retry and unknown push delivery.
- Staged rename verification expands both source and destination paths, avoiding false unsafe-state failures after Git folds delete/add into a rename.
- Unsupported conflict index shapes do not expose in-app resolution actions. Their index and `MERGE_HEAD` remain unchanged for external inspection.
- Integration records bounded preimages and the recovery ref before the checkout boundary. A simulated interrupted checkout followed by independent path/index edits yields `ExternalState`; the current index, edited path, untracked file and recovery copy remain unchanged. No reset, clean or automatic rollback is used.
- Capacity exhaustion is injected through the preimage byte limit, and journal persistence failure blocks protected mutation before it starts. Commit/checkout observer interruptions, external staging/path changes, unknown `index.lock`, missing background authentication, accepted-but-unacknowledged delivery and later remote verification all have deterministic tests.
- Background helper eligibility is fail-closed: Git Credential Manager and `cache` are eligible, `wincred` only on Windows, and `osxkeychain` only on macOS. `store`, `libsecret`, inline/custom commands and SSH background checks are rejected. Noninteractive requests disable terminal, GCM and askpass prompts; an authentication miss pauses the scheduler.

## Executed checks

| Check | Result |
| --- | --- |
| `cargo test -p markion-git-sync --no-fail-fast` | Passed: 48 unit, 3 fixture and 18 integration tests |
| `cargo test -p markion git_` | Passed: 2 library and 24 application tests |
| `cargo test -p markion autosave_` | Passed: 1 library and 5 application tests |
| `cargo test -p markion preferences_` | Passed: 14 library and 10 application tests, including the single bottom Git-section regression |
| `cargo check -p markion` | Passed |
| `cargo test --workspace --no-fail-fast` | Passed, including all root, workspace-member and doc tests |
| `cargo fmt --all -- --check` | Passed |
| `git diff --check` | Passed; line-ending conversion notices only |
| `openspec validate add-git-workspace-sync --strict` | Passed |

The application-specific matrix includes quick-start defaults and input isolation, foreign-tab repository selection, silent-save-off behavior, autosave drain/stale Git epoch rejection, local/network write admission, image-only cache invalidation, historical restore staleness, cancellation and quit reconciliation. The full workspace run also covers the ordinary file-tree, save, Save As, resource and export writer regressions.

## Windows application smoke

An earlier run in this change used an isolated temporary APPDATA/LOCALAPPDATA profile and a disposable notes repository with a local bare remote. Personal settings and project remotes were not changed.

- Opened the debug application and Sync sidebar in an approximately 1180 × 790 window.
- Ran Sync Now through the UI and verified a clean worktree, exactly one new snapshot commit and the expected Unicode Markdown bytes on the local remote.
- Entered and dismissed connection settings without changing document text or metrics.
- Opened recent history, commit paths and a Unicode diff, then closed normally.
- Confirmed the Windows Git child process is created without a visible console; path tests cover spaces, Chinese text and leading-dash names. Two-clone tests normalize and verify Windows line-ending behavior.

## Remaining native-platform gate

Task 12.5 remains unchecked. This Windows host cannot produce real macOS/Linux application, Keychain or SSH-agent smoke evidence, and the Windows smoke used a local bare remote rather than a live HTTPS helper or SSH agent. Platform-specific helper decisions are covered by the pure matrix above, but those tests do not substitute for native credential UI and agent smoke. No archive or release was performed.
