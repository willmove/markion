## Why

The Preferences panel currently looks like it exposes several persisted display settings, but most of those rows only show the current value and cannot be changed there. Dark themes also leave the in-window menu bar styled like a light theme, which hurts readability and makes the app chrome feel inconsistent.

## What Changes

- Make all existing non-theme Preferences rows interactive where the preference is already supported: focus mode, typewriter mode, code line numbers, Preview adaptive width, sidebar visibility, and sidebar tab selection.
- Render boolean preference values as button-like controls instead of plain `on` / `off` text so they read as editable settings.
- Reorder the Preferences panel so Language appears above Theme.
- Theme the in-window menu bar and dropdowns from the active theme palette, including dark themes.
- Non-goals: no new preference fields, no changes to the native OS menu implementation, and no changes to Markdown parsing or derived-state caching.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `theme-preferences`: Preferences panel rows become interactive controls, boolean values use button-like affordances, and Language is ordered before Theme.
- `chrome-platform`: The in-window menu bar and dropdowns adapt their foregrounds, backgrounds, borders, and active states to the current theme.

## Impact

- Affected code is expected in `src/main.rs` for Preferences panel rendering/actions and in-window menu styling.
- Existing preference persistence in `src/model.rs` and `src/storage/preferences.rs` should remain unchanged unless implementation reveals a missing round-trip.
- Existing localized labels in `src/i18n.rs` should be reused where possible; any new user-visible string must be added there.
