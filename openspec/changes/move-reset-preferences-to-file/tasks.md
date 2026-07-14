## 1. Planning

- [x] 1.1 Read the relevant `chrome-platform`, `theme-preferences`, and `ui-i18n` specs.
- [x] 1.2 Create the OpenSpec proposal, design, and chrome-platform delta for moving Reset Preferences.

## 2. Implementation

- [x] 2.1 Move the in-window File/Help menu placement so Reset Preferences appears immediately below Preferences in File and no longer appears in Help.
- [x] 2.2 Mirror the same placement in the native OS menu builder.

## 3. Verification

- [x] 3.1 Run `openspec validate move-reset-preferences-to-file`.
- [ ] 3.2 Run `cargo test`.
