## ADDED Requirements

### Requirement: Preferences SHALL expose a default-on slash-command popup toggle

Preferences General SHALL expose a localized actionable boolean control for showing quick commands when typing `/`, reflecting the current value and using the active theme. The setting SHALL default to enabled, apply immediately, and persist as `slash_commands_enabled` in `config.toml`. Missing or non-boolean values SHALL fall back to enabled while retaining other valid settings. Reset Preferences SHALL restore enabled behavior. Changing the setting SHALL NOT modify document text, version, selection, dirty state, undo/redo history, or shared derived Markdown caches.

#### Scenario: Disable and reload
- **WHEN** the user disables the slash-command popup in General preferences and restarts the app
- **THEN** the control remains disabled and automatic slash completion remains off

#### Scenario: Older or invalid configuration
- **WHEN** `slash_commands_enabled` is absent or contains a non-boolean TOML value
- **THEN** the popup setting defaults to enabled and other valid preferences still load

#### Scenario: Reset restores enabled
- **WHEN** the user confirms Reset Preferences after disabling the popup
- **THEN** the setting returns to enabled and that value is saved

#### Scenario: Toggle preserves the document
- **WHEN** the user changes the popup setting in either direction
- **THEN** document text, version, selection, dirty state, undo/redo history, and shared derived-cache identities remain unchanged
