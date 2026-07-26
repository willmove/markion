# Tasks

## 1. Shortcut Registry and Effective Bindings

- [x] 1.1 Replace `menu_shortcuts` in `src/app/mod.rs` with a registry of entries `{ id, default_binding, label_win, label_mac }` (kebab-case ids), keeping every current binding and label unchanged.
- [x] 1.2 Add `shortcut_overrides: BTreeMap<String, String>` to `AppPreferences` and a keystroke-string formatter (`secondary-shift-s` -> `Ctrl+Shift+S` / `Cmd+Shift+S`, named keys included) with unit tests.
- [x] 1.3 Add effective-binding + effective-label resolution helpers (override when present and parseable, else default) with unit tests.

## 2. Persistence

- [x] 2.1 Mirror overrides in `PreferencesFile` as a `[shortcuts]` table omitted when empty; round-trip in load/save.
- [x] 2.2 Drop unknown ids and unparseable keystrokes at load with a `tracing` log line; default applies.
- [x] 2.3 Confirm preferences reset clears overrides (default `AppPreferences` write) and cover with a test.

## 3. Live Rebinding

- [x] 3.1 Extract bootstrap's full `bind_keys` set into `bind_app_keys(cx, &overrides)` (core editing keys, file-tree keys, registry actions at effective bindings); call it at startup with loaded preferences.
- [x] 3.2 Add an app method that applies a shortcut change: update overrides, persist `config.toml`, `clear_key_bindings()`, `bind_app_keys(...)`.

## 4. Preferences Panel Shortcuts Tab

- [x] 4.1 Give `preferences_panel_view` a General / Shortcuts tab strip; keep General content unchanged; retain platform/category selection as tab state; widen the shell for the Shortcuts tab.
- [x] 4.2 Port the categorized shortcut list into the Shortcuts tab, rendering effective labels and an overridden-state marker with per-action reset.
- [x] 4.3 Implement capture mode (focus + `on_key_down`): assign on valid keystroke, reject bare printable keys and conflicts (reserved fixed bindings included) with localized inline feedback, cancel on Escape.
- [x] 4.4 Remove the standalone shortcut modal, its state/handlers, and the Help -> Keyboard Shortcuts items (in-window and native menus); rewire `ShowShortcuts` (F1) to open Preferences on the Shortcuts tab.

## 5. Menus and i18n

- [x] 5.1 Render in-window menu shortcut hints from effective bindings (curated label for defaults, formatted label for overrides).
- [x] 5.2 Key the i18n shortcut catalog by action id and resolve displayed combos from effective bindings for both platform previews.
- [x] 5.3 Add all new `Msg` strings (tab labels, capture prompt, conflict/invalid feedback, reset) in every supported language.

## 6. Verification

- [x] 6.1 Update existing shortcut-panel/menu-label tests to the new placement and add tests for override resolution, formatter, persistence round-trip, invalid-entry fallback, and conflict detection.
- [x] 6.2 Run `cargo fmt`, focused tests, then `cargo test` (root package); confirm derived-state caching code is untouched.
- [x] 6.3 Manually verify: remap a shortcut, restart, reset, conflict rejection, and that typing/file-tree keys still work after rebinding; check a light and a dark theme.
- [x] 6.4 Run `openspec validate customize-shortcuts-in-preferences` and reconcile archive order with `add-tabbed-shortcut-panel` (superseded modal presentation).
