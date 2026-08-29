## MODIFIED Requirements

### Requirement: Auto-save and recovery
The editor SHALL, after a period of inactivity while `[auto_save] enabled` is true, write a recovery snapshot for every dirty document tab (named or untitled). The inactivity interval SHALL come from `[auto_save] delay_secs` (default 5 seconds, minimum 1). When the tab has a filesystem path and `[auto_save] silent_save` is true (default), the editor SHALL then write the document to that path, retire the corresponding recovery snapshot on success, and clear dirty when no edits raced the write. When `silent_save` is false, the editor SHALL NOT write the original path on inactivity; the recovery snapshot SHALL remain, the tab SHALL stay dirty, and status feedback SHALL report a recovery save rather than a destination auto-save. Auto-save SHALL be fully disableable via `[auto_save] enabled = false` (no timer, no recovery, no silent write-back); that master switch remains configurable only through the config file. The Preferences panel SHALL expose controls for `silent_save` and `delay_secs` only. Manual Save and Save As SHALL remain unaffected by these preferences.

#### Scenario: Saved document auto-saves after the configured interval
- **WHEN** a named dirty document is inactive past the configured interval and `enabled` and `silent_save` are both true
- **THEN** the document is written to its file path, the recovery snapshot for that save is retired on success, and the status bar reports the destination auto-save

#### Scenario: Unsaved document writes a recovery copy
- **WHEN** an untitled dirty document is inactive past the configured interval and `enabled` is true
- **THEN** a recovery copy is written and offered for restoration on the next launch
- **AND** the tab remains dirty

#### Scenario: Silent save disabled keeps recovery only
- **WHEN** a named dirty document is inactive past the configured interval, `enabled` is true, and `silent_save` is false
- **THEN** a recovery snapshot is written or replaced
- **AND** the original file path is not modified
- **AND** the tab remains dirty
- **AND** the status bar reports a recovery save, not a destination auto-save

#### Scenario: Auto-save disabled by config
- **WHEN** `[auto_save] enabled = false` is set in `config.toml`
- **THEN** no auto-save or recovery copy is written on inactivity; manual save is unaffected

#### Scenario: Delay and silent_save are configurable from Preferences
- **WHEN** the user changes the silent-save toggle or the auto-save delay in Preferences → General
- **THEN** the new values persist in `[auto_save]` and apply to subsequent inactivity timers without requiring a restart
