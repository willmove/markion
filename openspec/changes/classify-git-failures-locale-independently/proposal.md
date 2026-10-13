## Why

Markion decides how to react to a failed Git command by searching Git's error text for English phrases:
- "couldn't find remote ref" means a missing sync branch;
- "Could not resolve" / "Failed to connect" means offline;
- "non-fast-forward" / "[rejected]" means another device pushed first;
- "authentication failed" / "could not read username" and similar mean sign-in is needed.

Markion starts Git without pinning the message language. Git's own messages are translated on Linux distributions and Homebrew installs whenever the user's locale is not English (e.g. `zh_CN.UTF-8`). For those users the reactions silently break:
- a missing branch becomes a generic failure;
- offline background checks are reported as errors instead of quietly backing off;
- sign-in failures do not offer a retry;
- a "someone else pushed first" rejection does not get its one automatic refresh-and-retry.

The same text matching is duplicated in four places, and they have already drifted apart:
- `engine.rs` (offline, missing ref, non-fast-forward);
- `auth.rs` (`classify_failure`);
- `background.rs` callers;
- the app's `authentication_message`.

Users also see raw Git output and internal debug names in the status bar, for example `Other("fatal: …")` or `Rewritten`.

On the reference Windows machine, Git for Windows printed English even under `LANG=zh_CN.UTF-8`, so the locale defect mainly affects Linux and macOS users. The duplicated matching and the raw messages affect every platform.

## What Changes

- **Pin Git's message language** for every process Markion starts (`LANGUAGE=en`, `LC_MESSAGES=C`), so machine-read text is stable. Path and content bytes are unaffected; character-type locale is left alone.
- **Prefer structured signals over prose**:
  - push results are read from `push --porcelain` per-ref status lines (already requested, never parsed);
  - when a fetch failure is still unclassified, one `ls-remote --exit-code` distinguishes a missing sync branch (exit code 2) from other failures.
- **One classifier** in `markion-git-sync` maps a failed command to a failure kind:
  - cancelled, timed out, offline/unreachable before connecting, connection interrupted during transfer;
  - authentication needed;
  - SSH host identity unknown, SSH host identity changed, encrypted key;
  - sync location not found or inaccessible;
  - missing sync branch;
  - upload rejected (behind / refused by the server);
  - repository locked by another program;
  - other.

  The engine, background checks, authentication handling and the app all use it, replacing the four ad-hoc matchers.
- **Typed outcomes**:
  - waiting-to-upload, uncertain-delivery and needs-attention outcomes carry the failure kind;
  - background-check problems carry a typed issue instead of a formatted string;
  - the status projection derives offline / authentication-needed / needs-attention from the kind, not from message text.
- **Definite non-delivery is not "uncertain"**. When a push fails before the sync location could accept it (unreachable, sign-in refused, host identity failure, location not found, explicit rejection), the commit is reported as waiting to upload. No doomed verification fetch is attempted. Interrupted or unclassified push failures keep today's verification and uncertain-delivery handling.
- **Localized, human failure messages**: each failure kind has a localized ordinary message with a next step. Redacted, bounded raw Git output appears only in Advanced Git details. The engine's own safety stops travel through the same outcome path and also get localized ordinary messages instead of English `Display` text:
  - target changed;
  - unsafe repository state;
  - stale plan;
  - unsafe remote history;
  - unsafe incoming paths;
  - no commit.
- Non-goals:
  - changing network timeouts or adding fast offline detection (separate follow-up);
  - Windows process-tree cancellation;
  - onboarding's address probe and clone error flows, owned by `fix-git-onboarding-remote-usability`;
  - in-app credential entry;
  - the app-side English precheck sentences in `run_git_operation` (a separate localization follow-up).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `git-sync`: adds requirements that Git failure handling is independent of locale and Git message language; that push failures distinguish definite non-delivery from uncertain delivery; and that ordinary failure feedback describes the classified cause in the interface language, with raw output limited to Advanced Git details.

## Impact

- `crates/git-sync`:
  - new `failure.rs` (classifier, push porcelain parser);
  - `process.rs` (message-language environment);
  - `engine.rs` (typed outcomes, non-fast-forward via porcelain, missing-branch fallback, non-delivery short-circuit, background issues);
  - `auth.rs` (`classify_failure` delegates);
  - `background.rs` (typed issues);
  - `model.rs` (`SyncOutcome` fields).
- `src/app/git_sync.rs`, `src/app/git_panel.rs`: kind-based connection state and retry offer, localized messages, raw detail in Advanced view; remove `authentication_message`.
- `src/i18n/git.rs`: one ordinary message per failure kind and per background issue, in all seven languages, covered by the existing Backup/sync localization requirement (no `ui-i18n` delta).
- **API**: `SyncOutcome` variants gain a field and `BackgroundFetchResult::ActionableError` changes payload type, so crate and app match sites and tests update together. No persisted data changes.
- Invariants: no typing-path or rendering work; `markion-git-sync` stays gpui-free.
