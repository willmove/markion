## Context

The Backup-and-Sync setup dialog (from unarchived `add-git-workspace-sync`, extended by `simplify-git-sync-setup-and-messages`) has three defects experienced with a real repository whose `origin` points at a wrong address:

- The advanced-section disclosure control ([`git_panel.rs:2879`](src/app/git_panel.rs)) always renders `GitMsg::Advanced` regardless of `setup.advanced`, while the three other disclosure controls in the same file (sync center, settings, preferences) already bind `if open { Hide… } else { … }` with dedicated i18n variants.
- `attach_origin` ([`setup.rs:242`](crates/git-sync/src/setup.rs#L242)) returns `RemoteExists("origin")` when an origin exists with a different URL — never replacing; and `run_git_onboarding` silently ignores the entered address whenever the existing origin already resolves a target. Net effect: no path exists to correct a sync address, and the raw English error leaks into a localized dialog.
- `validate_user_remote` ([`git_sync.rs:1845`](src/app/git_sync.rs)) enforces HTTPS/SSH syntax but only on the clone/initialize routes; connecting the current folder skips it, and nothing probes reachability, so typo'd addresses fail only at first push.

Constraints: the baseline spec forbids **implicit** remote replacement (explicit confirmation is the compliant path); setup already runs with foreground credentials for network operations; `CommandLimits` (15 s default timeout) and `CancellationToken` bound every git subprocess; `publish_first_branch` re-binds `branch.<name>.remote/merge` after an exact push and re-resolves the target, so a replaced origin URL continues correctly.

## Goals / Non-Goals

**Goals:**

- Correcting an existing sync address in the setup dialog, behind an explicit show-both-addresses confirmation.
- One validation pipeline for the entered address on all routes: syntax rejection, then bounded reachability probe with warn-and-confirm.
- Disclosure controls that label their state; all new copy localized.

**Non-Goals:**

- Changing an already-connected repository's address from sync settings (no address field there; later change if wanted).
- Remotes other than `origin`; provider sign-in; relaxing mixed-repository advanced gates.

## Decisions

### 1. Replacement is decided by a pre-check in the app, not by reacting to the engine error

The onboarding probe that already runs when the dialog opens (`open_git_onboarding` background probe) additionally captures the existing origin URL (`OnboardingService::origin_url`) into `GitOnboarding.existing_origin: Option<String>`. On submit, a pure decision function compares the entered address with `existing_origin`:

```
submit(entered address, existing_origin, confirm flags)
   |
   v
entered empty? -------------------> old behavior (use existing binding / ask for address)
   |
   v
[reject] validate_user_remote ---- fail --> inline localized error, stop
   |
   v
entered == existing_origin? ------ yes ---> proceed (no probe, no prompt)
   |
   v
[probe] ls-remote (15s cap, cancellable, foreground creds)
   |                                  fail --> confirm-unreachable state
   v                                           (continue-anyway / cancel)
[confirm] existing_origin is Some? -- yes --> confirm-reset state
   |                                           (shows both addresses,
   v                                            reset-sync-address / cancel)
run_git_onboarding(..., replace_remote = true|false)
```

*Why pre-check instead of reacting to the `RemoteExists` error:* the app decides UI states; the crate call becomes a single well-defined operation per attempt, errors stay strings as today, and the confirmation can show the current URL without extra round-trips. The `RemoteExists` error remains in the crate as the guard for unconfirmed attempts (defense in depth; tests keep it covered) but becomes unreachable from this dialog.

### 2. Crate API: `attach_origin` gains an explicit replace mode; new bounded probe

- `attach_origin(repository, remote, allow_replace: bool, cancellation)`: existing URL identical → no-op `Ok`; existing differs and `allow_replace` → `git remote set-url origin <url>`; existing differs and not allowed → `RemoteExists` (unchanged). `run_git_onboarding` forwards the app's confirmed flag.
- `OnboardingService::probe_remote(remote: &str, cancellation)` → runs `git ls-remote <url>` outside any repository with `network_request` (foreground credentials) and `CommandLimits` default 15 s timeout; `Ok`/`Err` is the entire signal — network, auth, and missing-repository failures all map to the same warn-and-confirm state (no error taxonomy; the user action is identical).

After a confirmed replacement the existing flow continues untouched: `publish_first_branch` pushes the exact commit and re-binds upstream against the new URL, then `resolve_sync_target` produces the new binding.

### 3. Confirmations are inline two-step states, not a new modal

`GitOnboarding` gains `confirm_unreachable: Option<String>` and `confirm_remote_reset: Option<(String, String)>` (current, new). While either is set, the dialog body renders the comparison/warning and the primary action becomes the localized confirm label ("Reset sync address" / "Use this address anyway"); a second click submits again with the corresponding flag, any other edit to the address field (or mode switch) clears both states. This matches the existing error-and-retry interaction pattern and avoids introducing a confirm-modal mechanism.

### 4. Disclosure label pair, same as the other three toggles

New i18n variant `HideAdvancedSetup` ("Hide advanced setup" etc.); the toggle at `git_panel.rs:2879` binds `if setup.advanced { HideAdvancedSetup } else { Advanced }`, mirroring lines 1697/2639/3043.

### 5. Copy: reuse and add minimally

Inline syntax rejection reuses the existing `SetupRemoteHelp` message. New `GitMsg` entries: the hide-advanced label, the reset comparison + confirm button, the unreachable warning + continue-anyway button — each translated in all seven languages; `Cancel` already exists. `RemoteExists` text itself is never shown in the dialog anymore.

### 6. Publication classifies the remote before pushing (post-review correction)

User testing of the replacement flow hit a dead end the design above did not cover: after a confirmed reset onto a remote that already contains work (the common "re-point my notes repo at the real GitHub repository" case), `publish_first_branch`'s exact non-fast-forward push is rejected by the server — with a misleading "check login access" hint and a byte-array debug dump of the git output.

`publish_first_branch` now fetches `--no-tags` first and classifies the remote tracking ref before pushing:

```
remote branch absent        -> exact first publication push (unchanged)
remote == local commit      -> bind upstream only
merge-base == remote        -> remote strictly behind: exact push fast-forwards it
merge-base exists otherwise -> remote ahead/diverged: bind upstream only;
                               the first Sync Now fetches, merges, and pushes
no merge-base               -> Err(UnrelatedRemoteHistory); the app shows the
                               localized clone-and-copy guidance instead of
                               the login retry hint
```

Deferring integration to the first sync reuses the engine's existing fetch/merge/push and conflict handling instead of duplicating merge logic in onboarding, and keeps the baseline rule of never auto-merging unrelated histories. Alongside this, `GitCommandOutput` gained a readable `Display` (lossy stderr/stdout text, 400-char cap) so failed commands stop rendering as byte arrays.

## Risks / Trade-offs

- [Probe adds up to 15 s latency and may trigger a system credential prompt for private repositories] → It runs only when a new address will actually be applied, in the existing background executor with the dialog's busy state and cancellation; foreground credential prompting is already the established onboarding behavior.
- [`set-url` changes a remote the user may share with other tools] → Exactly why it sits behind a two-step explicit confirmation that displays both addresses; cancelling changes nothing.
- [Probe requires network on first setup] → Warn-and-confirm rather than rejection keeps offline setup possible; only provably invalid syntax is refused outright.
- [String comparison of addresses may miss semantically equal URLs (trailing `.git`, casing)] → Acceptable: a redundant prompt is safe; the identical-address fast path is an optimization, not a correctness gate.

## Migration Plan

None. No persisted format changes; behavior is contained in the setup dialog. Rollback is a plain revert.

## Open Questions

- Final wording of the new localized strings per language — resolved during implementation following `src/i18n/git.rs` conventions.
