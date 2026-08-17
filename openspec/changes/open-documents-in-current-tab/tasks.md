## 1. Preference persistence and plumbing

- [x] 1.1 Add `open_in_current_tab: bool` (default `true`) to `AppPreferences` and its `Default` impl in `src/model.rs`
- [x] 1.2 Add the field to `PreferencesFile` with a new `deserialize_bool_or_true` helper (invalid → default on), both `From` impls, and round-trip plus missing/invalid-field tests in `src/storage/preferences.rs`
- [x] 1.3 Mirror `open_in_current_tab` on `MarkionApp` (`src/app/mod.rs`), load it in `MarkionApp::new` and write it in `current_preferences` (`src/app/application.rs`)
- [x] 1.4 Add `toggle_open_in_current_tab` in `src/app/appearance.rs` following the `toggle_show_hidden_files` pattern (flip, status message, `persist_preferences`, notify)
- [x] 1.5 Add `Msg` variants (Preferences label + status on/off) and translate them in all seven language blocks in `src/i18n.rs`
- [x] 1.6 Add the `preference_boolean_row` toggle to the Preferences General tab's "Other" section in `src/app/root_view.rs`

## 2. Default open-target resolution

- [x] 2.1 Implement `default_open_intent()` on `MarkionApp` per design D1/D2: preference off → `OpenInNewTab`; preference on → `ReplaceActive` only when the active tab is an image tab, an untitled document, or a clean document, otherwise `OpenInNewTab`
- [x] 2.2 Route the file-tree plain click and the file-tree context-menu Open action through `open_supported_path` with `default_open_intent()` (`src/app/application.rs` `open_tree_file`, `src/app/workspace.rs`)
- [x] 2.3 Route Open Recent through `default_open_intent()` instead of the hard-coded new-tab intent (`src/app/application.rs` `open_recent_path`)
- [x] 2.4 Route File → Open through `default_open_intent()`, keeping `confirm_discard_then` when the resolved intent is `ReplaceActive` and skipping it when it is `OpenInNewTab` (`src/app/documents.rs`)
- [x] 2.5 Multi-file drop: first supported file uses `default_open_intent()`, every subsequent file forces `OpenInNewTab` (`src/app/workspace.rs` `handle_external_drop`)

## 3. Ctrl/Cmd+click escape hatch

- [x] 3.1 Detect the platform modifier (Ctrl on Windows/Linux, ⌘ on macOS) in the file-tree row click handler and force `OpenPathIntent::OpenInNewTab`, leaving plain clicks on the preference-driven path (`src/app/root_view.rs`)
- [x] 3.2 Verify the file-tree context-menu "Open in New Tab" action still appends unconditionally

## 4. Tests and verification

- [x] 4.1 Tests for the intent matrix: preference on/off × active tab (image / untitled / clean / dirty) × already-open dedup precedence
- [x] 4.2 Test that a gesture open with a dirty active tab appends a new tab and leaves the dirty tab's text, undo history, and recovery snapshot untouched
- [x] 4.3 Test multi-file drop ordering: first file replaces a clean active tab, subsequent files append, last file is active
- [x] 4.4 Preferences tests: missing field → on, invalid value → on, round-trip, reset restores on
- [x] 4.5 `cargo test --workspace` green
- [ ] 4.6 Manual smoke pass: tree click on clean/dirty/welcome/image active tabs, Ctrl/Cmd+click, drag-drop single and multi, Open Recent, File → Open both preference states, toggle round-trip

## 5. Spec-tree alignment and docs

- [x] 5.1 Rewrite the unarchived `add-drag-drop-open` delta's drop-opening requirement to the preference-driven rule so both changes archive consistently
- [x] 5.2 Document the Ctrl/Cmd+click gesture next to the file-tree opening behavior in `docs/`
- [x] 5.3 `openspec validate open-documents-in-current-tab` passes
