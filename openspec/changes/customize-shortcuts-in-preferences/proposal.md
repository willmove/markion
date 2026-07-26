## Why

The keyboard-shortcut reference lives in a modal opened from Help -> Keyboard Shortcuts, where users must leave their settings context to read bindings, and every binding is hard-coded at compile time. Users expect shortcuts to live next to the other preferences and to be remappable without editing source code.

## What Changes

- Move the shortcut reference out of the Help menu into the Preferences panel as a dedicated "Shortcuts" tab next to the existing general settings tab. The standalone shortcut modal is removed; Help keeps only About.
- Make every menu-bound shortcut customizable from that tab: click a binding, press a new key combination, and the change applies immediately and persists to `config.toml` in a `[shortcuts]` override table keyed by stable action id.
- Menu items and the shortcut list always display the effective (possibly overridden) binding; invalid stored overrides fall back to defaults.
- Reject assignments that conflict with another action's effective binding, with localized inline feedback; per-action "reset to default" and inclusion in the global preferences reset.
- F1 keeps working: it now opens the Preferences panel directly on the Shortcuts tab.
- Non-goals: remapping core text-editing keys (arrows, backspace, enter, tab, selection), unbinding actions entirely, multiple bindings per action, shortcut search, and changes to file-tree-internal keys (F2/F5/etc.).

## Capabilities

### New Capabilities

- `keyboard-shortcuts`: Customizable menu-action shortcuts — override persistence in `config.toml`, effective-binding resolution with default fallback, live rebinding, conflict rejection, per-action and global reset, and capture-based editing UI.

### Modified Capabilities

- `chrome-platform`: The shortcut reference moves from the Help menu modal into a Preferences panel tab; Help retains only About; menu shortcut labels reflect effective customized bindings; preferences reset also clears shortcut overrides.
- `ui-i18n`: New localized strings for the Shortcuts tab, capture prompt, conflict feedback, and reset affordances in all supported languages.

## Impact

- Affected code: `src/app/mod.rs` (`menu_shortcuts`, panel state), `src/app/bootstrap.rs` (key binding, menus), `src/app/search.rs` (`ShowShortcuts` handler), `src/app/root_view.rs` (Preferences panel tabs, shortcut list UI, menu labels), `src/app/appearance.rs` (panel open/close), `src/i18n.rs` (catalog keyed by action id, new strings), `src/model.rs` and `src/storage/preferences.rs` (`[shortcuts]` override table).
- GPUI `clear_key_bindings` + full rebind on change; core editing keys are rebound unchanged so invariants hold.
- Markdown parsing and the per-document derived-state caches are not touched.
