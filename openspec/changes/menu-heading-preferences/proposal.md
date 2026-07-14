## Why

The Markdown formatting engine already supports ATX headings H1–H6, but the Format menu, native OS menu, and keyboard shortcuts previously exposed only H1–H3. Users who need H4 or H5 should not have to type markers manually, while H6 can remain an optional extra.

## What Changes

- Add a persisted **Heading menu depth** preference (default **H1–H5**, optional **H1–H6**) in `config.toml` and the Preferences panel.
- When set to H1–H6, expose H4, H5, and H6 in the in-window Format dropdown and native Format menu, with `Ctrl+4` / `Ctrl+5` / `Ctrl+6` shortcuts (platform `secondary-*` convention).
- When set to H1–H5 (default), menus and shortcuts expose H4 and H5 immediately.
- Changing the preference reapplies immediately, persists, participates in preferences reset, and reinstalls native menus so OS chrome stays in sync.
- Non-goals: setext headings, per-level customization beyond the depth toggle, changes to parsing/preview/outline caching, or moving non-heading Format actions into Preferences.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `markdown-editing`: Format menu and shortcuts SHALL reflect the configured heading depth; H4–H6 formatting actions become first-class when enabled.
- `theme-preferences`: Preferences panel gains an interactive Heading menu depth control with persistence and reset behavior.
- `ui-i18n`: New localized strings for H4–H6 menu labels, the preference control, shortcut reference entries, and related status text.
- `chrome-platform`: Narrow-scope preferences list and reset behavior include Heading menu depth.

## Impact

- `src/model.rs` — `AppPreferences` field and default.
- `src/storage/preferences.rs` — TOML round-trip, validation/defaulting.
- `src/main.rs` — MarkionApp state, Format/native menu builders, actions, keybindings, Preferences panel row, reset/persist paths, `install_menus` signature.
- `src/i18n.rs` — new `Msg` variants and translations.
- Focused unit tests in `src/storage/preferences.rs` and/or `src/main.rs` tests module.
- Does **not** touch derived Markdown caches, preview parsing, or editor typing-path performance invariants.
