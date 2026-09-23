# Proposal: simplify-git-setup-dialog-choices

## Why

User feedback on the Backup-and-Sync setup dialog: the route choice is hidden behind a "其他设置方式 / hide other setup options" disclosure and the dialog repeats the selected route's name as the bottom primary button, which reads as confirming a choice that was already made. Users see a heading, a toggle, three buttons, and a mode-named action — four representations of one decision. Presenting the three routes directly with the recommended one preselected, plus a plain Confirm/Cancel pair, matches the standard dialog affordance users expect.

## What Changes

- **Route selection is always visible**: the three setup routes (use this folder / obtain notes from an existing sync address / enable backup for this folder) render as one directly visible selectable row; the contextual recommendation is preselected exactly as today, the current selection is marked with a check, and switching routes keeps the existing field-default behavior.
- **The disclosure layer is removed**: the "other setup options / hide other setup options" toggle, the `alternatives_open` state, and the duplicated selected-route heading above the buttons are deleted.
- **Generic bottom actions**: the primary button becomes a plain "Confirm" (the mode-specific verb, the error-state "Retry" label, and the advanced-required "Advanced Repository Setup" label all collapse into it); the secondary stays "Cancel" with its Simplified Chinese wording aligned to "取消".
- Non-goals: no change to route recommendation logic, per-route fields, the inline address confirmations, advanced review gating, or automatic adoption; no new setup routes.

Note: this intentionally supersedes the not-yet-archived `add-git-workspace-sync` wording that alternatives stay "behind a secondary choice" — direct visibility with a preselected recommendation replaces the hidden-disclosure presentation; the change also reconciles that baseline delta.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `chrome-platform`: adds a requirement that setup route selection is directly visible with the contextual recommendation preselected and that the dialog's bottom actions are generic confirm/cancel.
- `ui-i18n`: adds a requirement covering the localized confirm label and the removal of the now-unused disclosure label pair.

Note: `chrome-platform` exists in main specs; the change is otherwise written against the unarchived `add-git-workspace-sync` + `fix-git-onboarding-remote-usability` baseline and must be archived after both.

## Impact

- `src/app/git_panel.rs` (`onboarding_view`): remove the selected-route heading and the alternatives toggle/button; render the three route buttons unconditionally; primary button label becomes the new confirm string.
- `src/app/git_sync.rs`: remove `alternatives_open` from `GitOnboarding` initialization; drop `toggle_git_onboarding_alternatives` and its callers.
- `src/i18n/git.rs`: add a `Confirm` variant (7 languages); remove `OtherSetupOptions`/`HideSetupOptions` variants and translations; align the Simplified Chinese `Cancel` wording to "取消".
- `src/app/tests.rs`: update the `GitOnboarding` construction after the field removal.
- `openspec/changes/add-git-workspace-sync/specs/git-workspace/spec.md`: reconcile the "alternative routes behind a secondary choice" wording with the directly-visible presentation (unarchived planning artifact, no main-spec edit).
- Invariants preserved: pure UI-layout change; no sync-engine, policy, or typing-path impact; all new/changed strings go through `src/i18n.rs`.
