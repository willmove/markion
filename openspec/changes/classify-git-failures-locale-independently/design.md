## Context

See proposal.md for motivation. Current classification sites, all matching stderr prose:

| Site | Matches | Drives |
|---|---|---|
| `engine.rs` `fetch_target` | `couldn't find remote ref` | missing target vs error |
| `engine.rs` `background_check` | `Could not resolve`, `Failed to connect`, `Connection timed out` | quiet offline backoff vs status error |
| `engine.rs` `is_non_fast_forward` | `non-fast-forward`, `[rejected]` | the one automatic push retry |
| `auth.rs` `classify_failure` | host-key / passphrase / `authentication failed` / `permission denied` / `could not read username` | background auth pause |
| `git_sync.rs` `authentication_message` | a superset of the above, applied to `reason` strings | retry offer, AuthenticationNeeded state |

`GitCommandRunner` sanitizes `GIT_*` variables but leaves locale variables untouched.

Observed on the reference machine (Git 2.51.1.windows.1):
- under `LANG=zh_CN.UTF-8` the messages stayed English, because that Git for Windows install does not translate;
- `push --porcelain` prints `!\t<src>:<dst>\t[rejected] (non-fast-forward)` on **stdout** and exits 1;
- curl/OpenSSH messages (e.g. `Could not resolve host`) are produced by those tools, not by Git's catalog.

Linux distribution and Homebrew builds ship Git catalogs, so Git's own prefixes and messages (`couldn't find remote ref`, `Authentication failed for`, `could not read Username`) are translated there.

`SyncOutcome::{AwaitingUpload, UncertainDelivery, NeedsAttention}` carry only `reason: String`. The app re-parses that string to decide state. `BackgroundFetchResult::ActionableError(String)` is built with `format!("{relation:?}")` / `format!("{failure:?}")`, and that string goes straight into the status bar.

## Goals / Non-Goals

**Goals:**
- One classifier, with typed results flowing from the engine to the presentation.
- Identical behavior across locales.
- Ordinary messages localized per kind.

**Non-Goals:**
- Shortening network timeouts.
- Job-object/process-group cancellation.
- Onboarding probe/clone messaging.
- In-app credential entry.

## Decisions

### 1. Pin Git's message language, not the whole locale

`GitCommandRunner::run` sets `LANGUAGE=en` and `LC_MESSAGES=C` on every child.
- GNU gettext gives `LANGUAGE` the highest priority whenever the message locale is not `C`. `LC_MESSAGES=C` covers the case where only `LANG` is set.
- If the user exports `LC_ALL`, `LC_MESSAGES` is ignored but `LANGUAGE=en` still wins. Since no `en` catalog exists, the untranslated (English) msgids are used.
- `LC_CTYPE`/`LC_ALL` are deliberately **not** overridden, so character handling in hooks, credential helpers and SSH stays as the user configured it.

Alternative considered: `LC_ALL=C`. Rejected, because it switches hooks and helpers to an ASCII character type. Alternative considered: `LC_ALL=en_US.UTF-8` (as some IDEs do). Rejected, because the locale may not be installed, and it also overrides the character type.

Pinning is a robustness layer, not the primary signal. Decisions 2 and 3 work even if a Git build ignores `LANGUAGE`.

### 2. Structured signals first

- **Process termination**: the runner's own `Cancelled` / `TimedOut` terminations map directly to kinds.
- **Push**: `push_exact` (and the conflict finish push) parse `--porcelain` stdout on success **and** failure into `PushRefStatus { flag, src, dst, summary, reason }`.
  - `!` + `[rejected]` with reason `non-fast-forward` or `fetch first` → `UploadRejected { behind: true }`, which drives the existing single retry;
  - any other `!` → `UploadRejected { behind: false }`;
  - no ref line at all → fall back to stderr classification.

  Ref-status summaries are fixed tokens in porcelain mode.
- **Missing sync branch**: if a fetch of the bound branch fails and the stderr classification yields `Other`, run one `git ls-remote --exit-code <remote> <destination_ref>`.
  - Exit code 2 → `MissingSyncBranch`;
  - exit code 0 → keep `Other`;
  - failure → classify that failure instead.

  The fallback is skipped when the first failure was already Offline/Interrupted/Authentication/host-key, so an offline device does not wait twice.

### 3. One classifier module

`crates/git-sync/src/failure.rs`:

```
pub enum GitFailureKind {
    Cancelled, TimedOut,
    Offline,                 // failed before a connection was established
    ConnectionInterrupted,   // connection lost after it was established
    AuthenticationNeeded,
    HostKeyUnknown, HostKeyChanged, EncryptedKey,
    LocationNotFound,        // 404 / "Repository not found" / "does not appear to be a git repository" / 403 no access
    MissingSyncBranch,
    UploadRejected { behind: bool },
    RepositoryLocked,        // index.lock exists / cannot lock ref
    Other,
}
pub struct GitFailure { pub kind: GitFailureKind, pub detail: String /* redacted, ≤ 400 chars */ }
pub fn classify(error: &GitCommandError) -> GitFailure;
pub fn parse_push_porcelain(stdout: &[u8]) -> Vec<PushRefStatus>;
```

Pattern table (case-insensitive; Git text is English after Decision 1; curl/OpenSSH text is never translated). Checked in this order:
1. **`HostKeyChanged`**: `remote host identification has changed`.
2. **`HostKeyUnknown`**: `no .* host key is known`, `authenticity of host`, `host key is not cached`, `host key verification failed`.
3. **`EncryptedKey`**: `enter passphrase for key`, `incorrect passphrase`, `bad passphrase`.
4. **`AuthenticationNeeded`**:
   - `authentication failed`, `could not read username`, `could not read password`, `terminal prompts disabled`;
   - `permission denied (publickey`;
   - `returned error: 401`, `http basic: access denied`.
5. **`LocationNotFound`**:
   - `repository not found`, `does not appear to be a git repository`;
   - `returned error: 404`, `returned error: 403`;
   - `permission to .* denied`.
6. **`Offline`** (pre-connection only):
   - `could not resolve host`, `could not resolve hostname`, `temporary failure in name resolution`, `name or service not known`, `nodename nor servname`;
   - `failed to connect to`, `connection refused`, `network is unreachable`, `no route to host`;
   - `connect to host .* connection timed out`.
7. **`ConnectionInterrupted`**: `connection reset`, `the remote end hung up unexpectedly`, `rpc failed`, `early eof`, `operation timed out`, `broken pipe`.
8. **`RepositoryLocked`**: `index.lock': file exists`, `unable to create '.*\.lock'`, `cannot lock ref`.
9. **`MissingSyncBranch`**: `couldn't find remote ref`.

Anything else is `Other`. `detail` reuses the existing redaction (`https://user@` → `https://[redacted]`), extended to `http://` and to `user:password@` forms, and is bounded like `GitCommandOutput`'s `Display`.

Every existing site calls this one function:
- `is_non_fast_forward` is deleted;
- `Authentication::classify_failure` maps `GitFailureKind` → `AuthFailure` for its remaining callers;
- the app's `authentication_message` / `sync_outcome_needs_authentication` are deleted.

### 4. Typed outcomes and background issues

- `SyncOutcome::{AwaitingUpload, UncertainDelivery, NeedsAttention}` gain `failure: Option<GitFailureKind>`. It is `None` for non-command causes, such as `verify_delivery` observing that the commit is absent.
- `SyncEngineError` gets `pub fn failure_kind(&self) -> Option<GitFailureKind>` for `Command` errors. The app's `unwrap_or_else` that turns engine errors into `NeedsAttention` fills `failure` from it.
- `BackgroundFetchResult::ActionableError(String)` becomes `ActionableError(BackgroundIssue)`:

  ```
  enum BackgroundIssue {
      TargetChanged, RepositoryNeedsAttention, NoLocalCommit,
      MissingSyncBranch, RemoteHistory(HistoryRelation), Failure(GitFailureKind),
  }
  ```

  The same change applies to `BackgroundNotification`. Mapping:
  - `Offline` and `ConnectionInterrupted` → `BackgroundFetchResult::Offline` (quiet backoff);
  - auth, host-key and encrypted-key kinds → `AuthenticationNeeded` (pause).

### 5. Definite non-delivery short-circuit

In `sync_now`'s and `push_commits`' push-error branches, classify first.
- **Definite non-delivery**: `Offline`, `AuthenticationNeeded`, `HostKeyUnknown`, `HostKeyChanged`, `EncryptedKey`, `LocationNotFound`, `UploadRejected { behind: false }`.
  - The checkpoint returns to a pending-upload phase (`NeedsAttention`, as the existing fetch-failure path does) instead of `Verifying`.
  - No verification fetch runs, and the outcome is `AwaitingUpload { failure: Some(kind) }`.
- **`UploadRejected { behind: true }`**: keeps the existing single refresh/integrate/retry, then `AwaitingUpload` if the retry budget is spent.
- **Everything else** (`ConnectionInterrupted`, `TimedOut`, `Cancelled`, `RepositoryLocked`, `Other`): keeps today's verification fetch and `UncertainDelivery`. `RepositoryLocked` stays uncertain because a local ref-lock failure can happen after the remote accepted the push.

### 6. Presentation and messages

`backup_sync_presentation` derives the connection state from typed facts:
- **AuthenticationNeeded**: the last outcome's or background result's kind is auth, host-key or encrypted-key. `retry_operation` is set from the same predicate.
- **Offline**: the kind is `Offline`, `ConnectionInterrupted` or `TimedOut`, or the background result is `Offline`.
- **NeedsAttention**: the kind is `LocationNotFound`, `MissingSyncBranch`, `UploadRejected` or `RepositoryLocked`, or there is a background issue.

Ordinary messages come from new `GitMsg` entries:
- one per `GitFailureKind`, with "local work is safe" wording where true and a next step;
- one per `BackgroundIssue`, with `RemoteHistory` split into rewritten / unrelated / deleted-target;
- one per `SyncEngineError` safety stop.

`GitMsg::Awaiting` / `Uncertain` receive the localized cause instead of the raw reason. The raw `detail` is shown only in Advanced Git details, through the existing advanced disclosure.

## Risks / Trade-offs

- **[A Git build ignores `LANGUAGE`]** → the porcelain, exit-code and `ls-remote` signals still classify the decision-critical cases: retry, missing branch, cancellation and timeout. Offline/auth/host-key detection relies on curl/OpenSSH text, which is never translated, or falls back to `Other`, which is safe: it shows a generic message and keeps uncertain-delivery verification.
- **[Pattern drift across Git, curl or OpenSSH versions]** → patterns live in one table with fixture tests per pattern. An unknown message degrades to `Other`, the pre-change behavior for most paths.
- **[Misclassifying an interrupted push as definite non-delivery]** → only pre-connection patterns are in `Offline`. Connection-loss text maps to `ConnectionInterrupted`, which stays uncertain. Fixtures include `RPC failed; curl 56` and `the remote end hung up unexpectedly` during push.
- **[English-pinned helper prompts on Linux]** → SSH askpass and Git-launched helpers show untranslated prompts. Accepted: correct classification matters more, and these prompts are mostly untranslated already.
- **[Public API change ripples into tests]** → all match sites are in this workspace and are updated together. `..` patterns keep most matches source-compatible.

## Migration Plan

No persisted data changes. Existing journals with `Verifying` checkpoints keep today's verification path on restart.
