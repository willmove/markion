# Tasks

## 1. Preference model and persistence

- [x] 1.1 Add the typed outline-position preference and tolerant TOML mapping; verify Left/Right round-trips and missing/invalid values preserve other preferences with storage tests.
- [x] 1.2 Wire startup, current-preference snapshots, and reset to the new setting; verify app preference persistence and reset behavior in GPUI tests without changing document caches.

## 2. General preferences and workspace layout

- [x] 2.1 Add localized Outline position Left/Right controls in General preferences; verify the controls apply and save the chosen side through pointer activation and labels exist in all supported languages.
- [x] 2.2 Dock the shared sidebar on the chosen side only for Outline and adapt its inner border/divider and drag coordinates; verify geometry, Files switching, hidden visibility, navigation/folding, resizing limits and default-width reset across document modes in GPUI tests.

## 3. Integration verification

- [x] 3.1 Run formatting, relevant regression tests, the root package test suite, and a build; record results and any remaining verification limits.
- [x] 3.2 Validate the OpenSpec change before archiving and confirm every tracked task is complete.

## Workflow follow-up

- Archive this completed change and sync its delta into the stable capability spec.
