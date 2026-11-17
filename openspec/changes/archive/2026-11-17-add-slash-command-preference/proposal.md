## Why

Typing `/` currently always offers the Visual Edit quick-command popup. Users need to disable this automatic popup in Preferences while retaining the current enabled behavior by default.

## What Changes

- Add a localized General preference for showing quick commands when typing `/`, enabled by default.
- Persist the boolean as `slash_commands_enabled` in `config.toml`; missing or invalid values and preference reset restore enabled behavior.
- Gate the existing slash popup, including its equivalent Chinese `、` trigger, and dismiss an open popup immediately when disabled.
- Preserve canonical document text, selection, history, dirty state, document version, and shared derived caches when changing the preference.

Non-goals: change the command catalog, popup layout, recognition contexts, emoji completion, or other formatting entry points.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `theme-preferences`: Expose and safely persist the default-on slash-command popup preference.
- `markdown-editing`: Make automatic slash completion conditional on the preference without changing literal input or document state.

## Impact

Touches `AppPreferences`, preference TOML conversion, app initialization/current preferences/reset, General settings UI and localization, and slash popup state synchronization. No new dependencies or changes to Markdown parsing or cached-per-version derivation are required.
