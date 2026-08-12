## Why

The file tree hides dotfile **directories** (`.git`, `.vscode`, …) but, because the skip predicate requires `path.is_dir()`, dotfile **files** like `.secret.md` leak through and appear in the tree. There is also no way for a user to reveal hidden entries when they actually want to — every editor explorer offers a "show hidden files" toggle and Markion does not. We need a single, consistent, preference-controlled definition of "hidden" (dotfile prefix on every platform, plus the Windows hidden file attribute on Windows) that the user can toggle from the Preferences panel, with the Markdown-only filter and the always-excluded build/dependency noise list left as separate concerns.

## What Changes

- A new boolean preference **Show hidden files/folders** (default **off**, preserving today's hide-dotfiles behavior) SHALL be added to the Preferences panel's non-theme display settings and persisted in `config.toml`.
- The file-tree scan SHALL treat "hidden" consistently for files **and** folders: an entry is hidden when its file name starts with `.` (every platform) **or**, on Windows, when the file carries the `FILE_ATTRIBUTE_HIDDEN` attribute. The preference controls only this OS-hidden layer.
- When the preference is **on**, hidden Markdown files and the folders that contain them SHALL appear in the tree subject to the existing Markdown-only filter; when **off**, hidden files and folders SHALL be omitted. Toggling SHALL re-scan the tree immediately and persist across restarts.
- The existing always-excluded build/dependency/VCS noise list (`target`, `node_modules`, `.git`, `.venv`, …) SHALL remain excluded regardless of the preference — it is a separate filtering layer, not the OS-hidden layer.
- As a deliberate consistency fix, dotfile **files** (e.g. `.secret.md`) that currently leak into the tree SHALL become hidden under the default-off preference, matching how dotfile directories already behave.
- New user-visible strings (Preferences panel label, optional on/off toggle status) SHALL be routed through the i18n layer for every supported language.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `workspace`: add a requirement that file-tree hidden-entry visibility is governed by a preference (dotfiles + Windows hidden attribute), with scenarios for default-hidden, toggle-on reveals, toggle-off re-hides, and persistence across restarts. Leaves the Markdown-only filter and the build/dependency noise list untouched.
- `theme-preferences`: add a requirement that the Preferences panel exposes a Show-hidden-files toggle in its display settings, that toggling applies immediately and re-scans the tree, and that the preference persists safely in `config.toml` (missing/invalid values default to off).
- `ui-i18n`: add a requirement routing the Show-hidden-files Preferences label and toggle status feedback through the i18n layer for every supported language, with exhaustiveness enforced at compile time.

## Impact

- `src/model.rs` (~line 166, 211) — add `pub show_hidden_files: bool` to `AppPreferences` and to its `Default` impl (default `false`).
- `src/storage/preferences.rs` (~line 29, 101, 135) — add `show_hidden_files: bool` to `PreferencesFile` with `#[serde(deserialize_with = "deserialize_bool_or_false")]`, and extend both `From` impls so the value round-trips.
- `src/storage/file_tree.rs` (~line 48, 260, 323-357) — thread the flag into the scan (e.g. `scan_with_options(root, show_hidden)` with `scan` kept as a thin `false` wrapper), and split `should_skip_file_tree_path` into (a) the always-on noise list and (b) the preference-gated OS-hidden check that now also covers dotfile **files** and the Windows hidden attribute (`cfg(windows)` + `MetadataExt::file_attributes()` & `0x2`).
- `src/app/application.rs` (~line 73, 611-632, 1056-1093) — add the field to `MarkionApp`, load it in `new`, snapshot it in `current_preferences`, and capture it in `schedule_file_tree_scan` so the background scan sees the live value.
- `src/app/appearance.rs` (~line 279) — add `toggle_show_hidden_files` mirroring `toggle_sync_scroll`, then call `self.refresh_file_tree(cx)` before `cx.notify()` so the tree re-scans under the new rule.
- `src/app/root_view.rs` (~line 3461-3527) — insert one `preference_boolean_row(...)` for the new toggle in the "Other settings" block.
- `src/i18n.rs` — add `Msg::PrefPanelShowHiddenFiles` (and optional on/off status variants) to the enum and to all eight `translate_*` arms.
- `src/app/tests.rs` — unit tests for the scan filter (hidden file hidden by default, revealed when on; noise list excluded in both states; Windows attribute path behind `cfg(windows)`) and a round-trip test for the preference.
- Preserves the invariants that the file tree renders a bounded number of rows per frame, that derived Markdown caches and syntax-highlighting memoization are untouched, and that the Markdown-only filter is unchanged. No GPUI dependency is introduced into any `crates/*` member.

## Non-goals

- Not changing the always-excluded build/dependency/VCS noise list or making it user-configurable.
- Not replacing `std::fs::read_dir` with the `ignore` crate or adding `.gitignore`/ignore-file support.
- Not revealing non-Markdown hidden files — the Markdown-only filter still applies, so only hidden Markdown files and the folders containing Markdown content can ever appear.
- Not adding per-entry "hide this" controls, a file-tree context-menu toggle, or workspace-scoped (per-root) visibility settings — the preference is global.
