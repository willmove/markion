## 1. Stable Git messages (`crates/git-sync`)

- [ ] 1.1 Set `LANGUAGE=en` and `LC_MESSAGES=C` for every child in `GitCommandRunner::run` without touching `LC_CTYPE`/`LC_ALL`; verify with a runner test that runs a shell alias (`git -c alias.e=!env e`) and asserts the child sees both variables while an `LC_ALL` value set for the test is passed through unchanged

## 2. Classifier (`crates/git-sync/src/failure.rs`)

- [ ] 2.1 Add `GitFailureKind`, `GitFailure` and `classify` with the ordered pattern table and redacted, bounded `detail`; verify with fixture tests covering at least one real message per kind:
  - curl DNS failure, OpenSSH `connect to host … Connection timed out`;
  - `RPC failed; curl 56`, `the remote end hung up unexpectedly`;
  - GitHub/GitLab `Repository not found`, HTTP 401/403/404;
  - changed/unknown host key, encrypted key, `index.lock` exists, `couldn't find remote ref`;
  - runner `Cancelled` / `TimedOut` terminations;
  - an unknown message → `Other`;
  - redaction of `https://user:token@host` and `http://user@host`
- [ ] 2.2 Add `parse_push_porcelain` returning per-ref status lines; verify with fixtures for an up-to-date push, a fast-forward, `[rejected] (non-fast-forward)`, `[rejected] (fetch first)`, `[remote rejected] (pre-receive hook declined)`, and output with no ref line

## 3. Engine integration

- [ ] 3.1 Add `failure: Option<GitFailureKind>` to `SyncOutcome::{AwaitingUpload, UncertainDelivery, NeedsAttention}` and `SyncEngineError::failure_kind`; update all crate and app match sites; verify `cargo test --workspace` compiles and existing sync-engine tests pass unchanged in behavior
- [ ] 3.2 Replace `is_non_fast_forward` with porcelain parsing in `push_exact`, and classify fetch failures in `sync_now`, `pull_updates` and `check_remote`; verify with a sync-engine test where another clone pushes first: exactly one automatic refresh/integrate/retry happens, with Git messages forced to a non-English catalog when available (skipped otherwise) and always with a fixture-injected rejection
- [ ] 3.3 Add the `ls-remote --exit-code` missing-branch fallback for unclassified fetch failures, skipped for Offline/Interrupted/auth/host-key kinds; verify with a sync-engine test that deleting the bound remote branch yields a missing-sync-branch result and that an unreachable remote does not trigger the fallback (counter or timing-free assertion via the classifier path)
- [ ] 3.4 Implement the definite-non-delivery short-circuit in `sync_now` and `push_commits` push-error branches; verify with sync-engine tests that:
  - an unreachable push URL returns `AwaitingUpload { failure: Some(Offline) }` with no verification fetch and a non-`Verifying` checkpoint;
  - a pre-receive hook rejection returns `AwaitingUpload { failure: Some(UploadRejected { behind: false }) }`;
  - an injected interrupted-transfer failure still returns `UncertainDelivery`
- [ ] 3.5 Change `BackgroundFetchResult::ActionableError` and `BackgroundNotification::ActionableError` to carry `BackgroundIssue`, and use the classifier in `background_check`; verify with background tests that Offline/Interrupted map to quiet backoff, auth kinds pause checks, a deleted branch yields `MissingSyncBranch`, and rewritten history yields `RemoteHistory(Rewritten)`
- [ ] 3.6 Make `Authentication::classify_failure` delegate to the classifier while keeping its `AuthFailure` mapping; verify the existing `ssh_security_failures_remain_distinct` test and add cases for 401 and `could not read Username`

## 4. App integration (`src/app`)

- [ ] 4.1 Fill `failure` in `run_git_operation`'s engine-error-to-`NeedsAttention` conversion, classify conflict finish/push command errors the same way, and delete `authentication_message` / `sync_outcome_needs_authentication` in favor of kind predicates for `retry_operation`; verify with app tests that an auth-kind outcome offers retry and an offline-kind outcome does not
- [ ] 4.2 Derive `ConnectionState` (AuthenticationNeeded / Offline / NeedsAttention) from typed outcome and background facts in `backup_sync_presentation`; verify with app tests for each kind group and for a `MissingSyncBranch` background issue
- [ ] 4.3 Replace raw `reason` / `{:?}` text in status and Backup and Sync feedback with localized per-kind, per-issue and per-`SyncEngineError` messages, and show the redacted `detail` only inside Advanced Git details; verify with app tests that no ordinary status contains `Other(`, `Rewritten` or raw `fatal:` text for each outcome path, and that Advanced details include the detail

## 5. Localization

- [ ] 5.1 Add `GitMsg` entries for every `GitFailureKind`, every `BackgroundIssue` (with rewritten / unrelated / deleted-target variants) and every `SyncEngineError` safety stop, in all seven languages, with "work is saved on this computer" wording where true; verify with the i18n completeness test

## 6. Verification

- [ ] 6.1 Run `cargo test --workspace` and confirm it passes
- [ ] 6.2 Manual check on Linux with `LANG=zh_CN.UTF-8` and the distribution's Git translations installed:
  - disconnect the network and run Sync Now: localized "saved on this computer, waiting to upload", not uncertain;
  - push from a second clone first, then Sync Now: one automatic retry succeeds;
  - delete the remote branch: missing-sync-branch message;
  - enable background checks while offline: no error status.

  Record results in this change folder
