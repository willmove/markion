## 1. Isolate test construction from the real preferences file

- [x] 1.1 In `MarkionApp::new`, under `cfg!(test)`, load `AppPreferences::default()` instead of `load_app_preferences(&default_preferences_path())`, matching the existing `session.toml` skip. Keep `preferences_path` as `default_preferences_path()` so production path identity is unchanged.
- [x] 1.2 Confirm a GPUI test that constructs `MarkionApp` without overriding `preferences_path` reports `DEFAULT_EDITOR_FONT_SIZE` (14) and other documented preference defaults, regardless of the developer machine's `config.toml`.

## 2. Guard preference writes in tests

- [x] 2.1 In `persist_preferences`, under `cfg!(test)`, return without writing when `self.preferences_path` equals `default_preferences_path()`. Do not no-op writes to other paths.
- [x] 2.2 Confirm existing tempfile persist tests (`typography_changes_preserve_document_caches_and_list_positions`, `non_default_editor_font_reflows_wrapped_text_and_caret_geometry`, and other tests that already set `preferences_path` to a tempfile) still write their isolated `config.toml`.

## 3. Close the known leak and add a regression

- [x] 3.1 Redirect `source_layout_snapshot_maps_wrapped_utf8_content_bidirectionally` to a tempfile `preferences_path`, the same way the neighboring 24px/32px typography tests already do, before it calls `set_editor_font_size(28)`.
- [x] 3.2 Add a GPUI regression test that constructs `MarkionApp` without overriding `preferences_path`, snapshots the real `config.toml` bytes (or its absence), calls `set_editor_font_size(28)`, and asserts the real file is unchanged.

## 4. Developer config recovery

- [x] 4.1 Confirm this machine's `%APPDATA%\Markion\config.toml` has `editor_font_size = 14` (restored from the leaked 28). Do not ship a production migration that rewrites other users' font sizes.
- [x] 4.2 After 3.1–3.2, re-run the former leaking test and the new regression and confirm the real `config.toml` still has `editor_font_size = 14`.

## 5. Automated verification

- [x] 5.1 Run `cargo fmt --check` and resolve any formatting differences introduced by this change.
- [x] 5.2 Run the new isolation/regression tests, then `cargo test` for the root package, without altering unrelated user changes.
- [x] 5.3 Run `openspec validate isolate-tests-from-user-config` and resolve every proposal/spec/design/task consistency error.
