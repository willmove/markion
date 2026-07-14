## MODIFIED Requirements

### Requirement: Auto-save and recovery
The editor SHALL auto-save after a period of inactivity, write saved documents to their file path, and write unsaved documents to a recovery copy that can be restored on the next launch. The inactivity interval SHALL come from the `[auto_save] delay_secs` config value (default 5 seconds) and auto-save SHALL be disableable via `[auto_save] enabled = false`; both are configurable only through the config file, not the Preferences panel.

#### Scenario: Saved document auto-saves after the configured interval
- **WHEN** a saved document is modified and the user is inactive past the configured auto-save interval
- **THEN** the document is written to its file path and the status bar reports the auto-save

#### Scenario: Unsaved document writes a recovery copy
- **WHEN** an unsaved document is modified and the user is inactive past the configured auto-save interval
- **THEN** a recovery copy is written and offered for restoration on the next launch

#### Scenario: Auto-save disabled by config
- **WHEN** `[auto_save] enabled = false` is set in `config.toml`
- **THEN** no auto-save or recovery copy is written on inactivity; manual save is unaffected
