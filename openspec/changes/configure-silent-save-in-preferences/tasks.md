## 1. Model and persistence

- [x] 1.1 Add `silent_save: bool` (default `true`) to `AutoSavePreferences` in `src/model.rs` and thread it through `AppPreferences` defaults / equality used by tests
- [x] 1.2 Persist `silent_save` under `[auto_save]` in `src/storage/preferences.rs` (omit → `true`); extend save/parse/reset round-trip tests and any preferences-summary coverage that lists auto-save fields

## 2. Autosave runtime gate

- [x] 2.1 Add `silent_save` to `AutosaveRequest`; in `run_autosave`, after a successful recovery write, skip destination save when `silent_save` is false and return `RecoveryOnly` (keep recovery file)
- [x] 2.2 Confirm `apply_autosave_outcome` leaves named `RecoveryOnly` tabs dirty with `last_recovery_file` set and uses recovery status messaging (not destination auto-saved)
- [x] 2.3 Add tests: `silent_save = false` on a named dirty tab writes recovery and does not modify the original path; `silent_save = true` still write-back + retires recovery; `enabled = false` still schedules nothing

## 3. Preferences General UI

- [x] 3.1 Add General-tab Auto-save section: silent save-to-file toggle and `delay_secs` stepper (min 1; max 300), mirroring existing preference control idioms
- [x] 3.2 Wire setters that update `auto_save_preferences`, call `persist_preferences()`, and apply without restart; ensure `enabled` has no panel control
- [x] 3.3 Include silent_save / delay_secs in preferences reset and Help summary paths where other General settings appear

## 4. Localization and docs

- [x] 4.1 Add Msg variants + translations for Auto-save section / silent-save / delay labels (and any toggle status text) in all supported languages; extend the exhaustive i18n catalog test
- [x] 4.2 Update `docs/faq.md` (and any config-schema mention) for `silent_save`, panel controls, and file-only `enabled`

## 5. Verification

- [x] 5.1 Run relevant unit/GPUI tests for preferences + autosave; fix failures
- [x] 5.2 Run `openspec validate configure-silent-save-in-preferences` and confirm the change is spec-consistent
