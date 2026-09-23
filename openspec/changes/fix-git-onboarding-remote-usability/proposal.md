# Proposal: fix-git-onboarding-remote-usability

## Why

Three usability defects in the Backup-and-Sync setup dialog block or mislead users whose folder already contains a Git repository with an origin remote:

1. The advanced-section disclosure control always reads "Advanced setup" — even when the section is expanded — so users cannot tell the current state.
2. An existing (possibly wrong) sync address cannot be corrected: when the origin URL is broken, submitting a new address fails with the unlocalized error "repository already has a remote named origin; it was not replaced"; when the origin URL is fine, the entered address is silently ignored. There is no path to replace the sync address at all.
3. Address validation only runs on the clone and initialize routes; connecting the current folder accepts unvalidated input, and no route checks reachability before the first publication, so a typo'd but syntactically valid address surfaces only as a late push failure.

## What Changes

- **State-reflecting disclosure label**: the advanced-section toggle shows a hide-advanced label while expanded and the advanced label while collapsed, matching the pattern already used by the other disclosure controls in the sync surfaces.
- **Explicitly confirmed sync-address replacement**: when the user submits an address that differs from the repository's existing origin URL, the setup dialog shows both addresses and requires a second explicit confirmation ("reset sync address"); after confirmation the origin is updated with `git remote set-url` and setup continues. An identical address proceeds without prompting. Implicit replacement remains forbidden, preserving the existing sync-binding safety requirement. The entered address is now honored on the connect-current-folder route (previously silently ignored when the existing origin resolved).
- **Unified address validation on every route**: unparseable addresses, non-HTTPS/SSH transports, embedded credentials, and local paths are rejected inline before anything touches the repository — including the connect-current-folder route that previously skipped validation.
- **Reachability probe with warn-and-confirm**: before applying a newly entered address, a bounded, cancellable `git ls-remote` probe checks it. When the address cannot be contacted (typo, missing repository, no access, or offline), the dialog warns and requires explicit confirmation to continue rather than failing silently or rejecting outright — offline setup stays possible.
- **Localized setup feedback**: all new confirmation and warning copy, and the replacement flow's messages, go through the i18n layer in every supported language; raw English engine errors no longer surface in this dialog.
- Non-goals: changing the sync address of an already-connected repository from sync settings (no address field exists there today); managing remotes other than `origin`; provider sign-in or hosted-repository creation; relaxing the existing advanced-review gates for mixed repositories.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `git-workspace`: adds a requirement that an existing sync address is replaceable through explicit confirmation with both addresses shown, and a requirement that sync addresses are validated (syntax rejection plus reachability warning) on every setup route.
- `chrome-platform`: adds a requirement that setup disclosure controls reflect their expanded/collapsed state in their label.
- `ui-i18n`: adds a requirement covering the localized copy for the disclosure label pair, the replacement confirmation, and the reachability warning.

Note: `git-workspace` is a capability introduced by the completed, not-yet-archived `add-git-workspace-sync` change; this change's deltas are written against that baseline (which itself sits on `simplify-git-sync-setup-and-messages`) and must be archived after both.

## Impact

- `crates/git-sync/src/setup.rs`: `attach_origin` gains an explicit replace mode (`git remote set-url`) with the remote-exists error retained for unconfirmed attempts; a new bounded `ls-remote` reachability probe on `OnboardingService`; crate tests for both.
- `src/app/git_sync.rs`: submit precheck validates the address on all routes; the onboarding probe captures the existing origin URL; `GitOnboarding` gains confirmation states (address replacement, unreachable address) and `run_git_onboarding` takes the confirmed-replacement flag; app-level tests for the precheck decision seam.
- `src/app/git_panel.rs`: disclosure toggle label binding; rendering of the two inline confirmation states.
- `src/i18n/git.rs`: new `GitMsg` entries (hide-advanced label, replacement compare/confirm, unreachable warning, continue-anyway) in all seven languages.
- Invariants preserved: all probes run in background executors with cancellation; no work on the typing/render path; `markion-git-sync` stays gpui-free; policy schema unchanged; baseline safety rule that remote configuration is never replaced implicitly is kept — replacement happens only after explicit user confirmation.
