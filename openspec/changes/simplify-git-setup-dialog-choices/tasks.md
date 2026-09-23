## 1. Route row and generic actions

- [x] 1.1 Rework `onboarding_view` in `src/app/git_panel.rs`: delete the selected-route heading block and the alternatives disclosure button; render the three route buttons unconditionally with the existing checkmark selection mark; replace the `primary` three-way label with a single new `GitMsg::Confirm`; verify the submit/cancel handlers and busy gating are unchanged
- [x] 1.2 Remove `GitOnboarding.alternatives_open`, `toggle_git_onboarding_alternatives`, and every remaining reference; update the `GitOnboarding` construction in `src/app/tests.rs`; verify `cargo test --bin markion` compiles and passes

## 2. Localized strings

- [x] 2.1 In `src/i18n/git.rs`: add the `Confirm` variant with all seven translations, remove `OtherSetupOptions`/`HideSetupOptions` and their translation blocks, and align the Simplified Chinese `Cancel` wording to "取消"; verify the i18n completeness and placeholder tests pass

## 3. Baseline reconciliation and verification

- [x] 3.1 Update the unarchived `add-git-workspace-sync` git-workspace delta wording so alternative routes are described as directly visible with the contextual recommendation preselected (superseding "behind a secondary choice"), and note the supersession in that change's proposal impact; verify `openspec validate add-git-workspace-sync` still passes
- [x] 3.2 Run `cargo test --workspace` and `openspec validate simplify-git-setup-dialog-choices`; both must pass
- [ ] 3.3 Manual smoke: open the setup dialog on (a) a dedicated notes repository, (b) an ordinary folder with notes, and (c) an empty folder; confirm three visible routes with the right one preselected, switching works, the bottom buttons read confirm/cancel, and the address confirmation flows from `fix-git-onboarding-remote-usability` still behave
