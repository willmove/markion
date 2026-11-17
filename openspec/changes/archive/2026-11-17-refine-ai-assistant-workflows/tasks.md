## 1. Setup completion

- [x] 1.1 Separate discovery validation and complete save-and-enable with identity-bound verification; verify blank-model discovery, persistence failure and matching capability integration tests.
- [x] 1.2 Add writing parameter feedback and localized workflow/state labels; verify all seven language catalog entries and no provider request for missing parameters.

## 2. Input and panel

- [x] 2.1 Implement cached soft-wrapped visual rows with caret/selection/IME mapping; verify CJK/emoji, newline and narrow-width GPUI input tests.
- [x] 2.2 Reorganize fixed header/composer, flexible message body, compact icon actions and contextual writing/workspace controls; verify small-window and multi-language/theme GPUI layouts and keyboard controls.
- [x] 2.3 Clarify context/history and profile switching and protect deletion; verify pending versus used sources, history state and confirmation tests.

## 3. Settings and review

- [x] 3.1 Group settings, show credential/verification state and units and keep save footer visible; verify draft guards, setup and settings layout tests.
- [x] 3.2 Show bounded writing additions/removals and target information; verify review summary and existing atomic undo/stale-target tests.

## 4. Acceptance

- [x] 4.1 Update bilingual guides and record delivered versus deferred improvements and native verification limits in a verification artifact; verify documentation matches the final interface.
- [x] 4.2 Run formatting, clippy, workspace tests, build and strict OpenSpec validation; fix regressions and record outcomes.

## 5. Reported regressions

- [x] 5.1 Restore direct printable-key input without leaking document shortcuts; cover Latin letters, shifted characters, punctuation, spaces, settings fields and existing IME/Undo isolation.
- [x] 5.2 Identify local attachment/request input-budget failures separately from generation limits and link to the existing advanced input limit; verify larger configured budgets accept complete attachments without truncation, and document the 1 MiB ceiling and explicit selection workaround.
- [x] 5.3 Run AI UI/core regression tests, formatting, clippy, build and strict change validation; verify native physical-key typing and record results without sending cloud requests or changing user documents.
