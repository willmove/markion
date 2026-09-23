## 1. Disclosure label reflects state (P1)

- [x] 1.1 Add the `HideAdvancedSetup` variant to `GitMsg` in `src/i18n/git.rs` with translations for all seven languages and bind the onboarding advanced toggle in `src/app/git_panel.rs` (line ~2879) to `if setup.advanced { HideAdvancedSetup } else { Advanced }`, mirroring the other three disclosure controls; verify the i18n completeness test passes and the toggle label flips with the section state in a manual run

## 2. Crate: replaceable origin and reachability probe

- [x] 2.1 Change `attach_origin` in `crates/git-sync/src/setup.rs` to take `allow_replace: bool`: identical existing URL stays a no-op, `allow_replace` with a differing URL runs `git remote set-url origin <url>`, and unconfirmed replacement still returns `RemoteExists`; verify crate tests cover no-op, replace, and refusal
- [x] 2.2 Add `OnboardingService::probe_remote(remote, cancellation)` running a bounded `git ls-remote` (default `CommandLimits` timeout, foreground credentials) with any failure mapped to one error; verify a crate test probes a local bare remote successfully and a unit test covers the invalid-address fast failure
- [x] 2.3 Update the `run_git_onboarding` signature in `src/app/git_sync.rs` to forward the confirmed replacement flag to `attach_origin`; verify existing callers compile and tests pass

## 3. App: unified validation, probe, and two-step confirmations

- [x] 3.1 Capture the existing origin URL in the onboarding background probe (`open_git_onboarding`) into a new `GitOnboarding.existing_origin: Option<String>`; verify the probe still respects its generation guard and stores the URL for the same workspace
- [x] 3.2 Add a pure decision seam `remote_submission_decision(existing_origin, entered, confirmed_reset, confirmed_unreachable) -> RemoteSubmissionAction` (Proceed / RejectSyntax / NeedsUnreachableConfirm / NeedsResetConfirm) in `src/app/git_sync.rs` with unit tests covering each branch including the identical-address fast path
- [x] 3.3 Rewire `submit_git_onboarding`: run `validate_user_remote` for a nonempty address on every mode (drop the clone/initialize-only gate), apply the decision seam, and background-spawn the reachability probe only when a new address will be applied; on NeedsUnreachableConfirm/NeedsResetConfirm set the new `GitOnboarding` confirmation states instead of spawning the operation; verify app-level tests for the precheck routing
- [x] 3.4 Render the two inline confirmation states in `src/app/git_panel.rs` (unreachable warning with continue-anyway, address comparison with reset-sync-address), clear both states when the address field or mode changes, and submit the confirmed flags on the second click; verify the busy/cancellation guards still apply while probing

## 4. Localized copy

- [x] 4.1 Add the new `GitMsg` entries (hide-advanced label, reset comparison and confirm button, unreachable warning and continue-anyway button) for all seven languages in `src/i18n/git.rs`, reusing `SetupRemoteHelp` for inline syntax rejection and the existing `Cancel`; verify the i18n completeness and placeholder tests pass

## 5. End-to-end verification

- [x] 5.1 Run `cargo test --workspace` and `openspec validate fix-git-onboarding-remote-usability`; both must pass
- [x] 5.2 Manual smoke on a scratch clone whose origin is set to a wrong URL (the screenshot scenario): open Backup and Sync setup, enter the correct address, confirm the unreachable/reset confirmations appear as designed, complete setup, and verify `git remote -v` shows the corrected address and a sync succeeds; also confirm the advanced toggle label flips with the section state

## 6. Publication after replacement handles non-empty remotes (post-review fix)

- [x] 6.1 Make `GitCommandError`'s `Failed` variant render stdout/stderr as readable (lossy, bounded) text instead of byte-array debug dumps; verify crate tests still compile and any Display assertions are updated
- [x] 6.2 Rework `publish_first_branch` in `crates/git-sync/src/setup.rs` to fetch and classify the remote branch before pushing: an absent remote branch keeps the exact first publication; a remote already at the local commit binds upstream only; a remote strictly behind still fast-forwards via the exact push; a remote ahead or diverged with a common ancestor binds upstream without pushing so the first sync integrates; histories with no common ancestor return a new `UnrelatedRemoteHistory` error; verify crate tests cover each classification against real bare remotes
- [x] 6.3 Map `UnrelatedRemoteHistory` in `run_git_onboarding` to a new localized `GitMsg::SetupUnrelatedRemote` guidance string (7 languages) instead of the login retry hint, which stays on transport/auth-style failures only; verify app tests and the i18n completeness test pass

## 7. End-to-end verification of the replacement flow

- [x] 7.1 Extend the wrong-origin integration test: after the confirmed replacement to a remote that already contains a related diverged commit, onboarding binds upstream without pushing and a subsequent engine sync fetches, merges, and pushes successfully; verify `cargo test --workspace` and `openspec validate fix-git-onboarding-remote-usability` pass
- [ ] 7.2 Manual smoke rerun of the screenshot scenario: reset the address to `git@github.com:willmove/spec-wiki.git`, confirm setup completes without the non-fast-forward error (the first sync merges the remote's existing work), and `git log` shows the merge
