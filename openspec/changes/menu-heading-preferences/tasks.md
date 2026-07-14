## 1. Preference model and persistence

- [x] 1.1 Add `heading_menu_max_level: u8` to `AppPreferences` (default `5`; normalize to `5` or `6` only).
- [x] 1.2 Round-trip the field through `src/storage/preferences.rs` (`PreferencesFile`, load/save, invalid → `5`).
- [x] 1.3 Add focused persistence tests (missing key, value `6`, invalid value).

## 2. Heading actions and menus

- [x] 2.1 Add `Heading4`, `Heading5`, `Heading6` actions and a shared `apply_heading_level` helper in `src/main.rs`.
- [x] 2.2 Refactor in-window Format dropdown heading items to build from `1..=heading_menu_max_level`.
- [x] 2.3 Extend `install_menus` to accept heading depth and emit H4–H6 when depth is `6`; update all call sites.
- [x] 2.4 Register `secondary-4/5/6` keybindings and wire action handlers.

## 3. Preferences panel and app state

- [x] 3.1 Load/save/reset `heading_menu_max_level` on `MarkionApp` (including `current_preferences` and reset path).
- [x] 3.2 Add Heading menu depth segmented control to the Preferences panel; on change persist and reinstall native menus.

## 4. Localization and verification

- [x] 4.1 Add `Msg` variants and en/zh strings for H4–H6 labels, preference control, and conditional shortcut reference lines.
- [x] 4.2 Run `openspec validate menu-heading-preferences` and `cargo test`.
