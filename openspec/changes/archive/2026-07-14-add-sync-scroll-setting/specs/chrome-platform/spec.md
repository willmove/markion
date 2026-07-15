## ADDED Requirements

### Requirement: Sync scroll preference
The editor SHALL provide a persisted "Sync scroll" preference, disabled by default, that when enabled and the active view mode is Split Preview SHALL couple the source editor and rendered preview scroll positions proportionally: scrolling either pane (mouse wheel, trackpad, or dragging that pane's scrollbar) SHALL move the other pane to the same fraction of its scrollable range, clamped to its bounds. The preference SHALL have no effect in Edit or Read mode, where only one pane is visible. The proportional coupling SHALL be based on each pane's scroll offset divided by its maximum scrollable offset, and SHALL be a no-op for a direction whose pane has no scrollable range. The coupling SHALL NOT force a Markdown reparse, reset the preview list, or disturb the per-version derived-state caches.

#### Scenario: Sync scroll defaults to off
- **WHEN** the editor starts with no `sync_scroll` value in the preferences file
- **THEN** Sync scroll is disabled and the source editor and preview panes scroll independently as before

#### Scenario: Scrolling the editor moves the preview proportionally
- **WHEN** Sync scroll is enabled, the active view mode is Split Preview, and the user scrolls the source editor pane
- **THEN** the rendered preview pane scrolls to the same fraction of its scrollable range as the source editor's current fraction

#### Scenario: Scrolling the preview moves the editor proportionally
- **WHEN** Sync scroll is enabled, the active view mode is Split Preview, and the user scrolls the rendered preview pane
- **THEN** the source editor pane scrolls to the same fraction of its scrollable range as the preview's current fraction

#### Scenario: Sync scroll is inactive outside Split Preview
- **WHEN** Sync scroll is enabled but the active view mode is Edit or Read
- **THEN** scrolling the visible pane does not affect any other pane and the preference persists without error

#### Scenario: A pane with no scrollable range does not drive the other
- **WHEN** Sync scroll is enabled, the view mode is Split Preview, and one pane's content fits within its viewport (no scrollable range)
- **THEN** scrolling that pane does not move the other pane, and the other pane may still scroll independently

### Requirement: Sync scroll preference persistence
The editor SHALL persist the Sync scroll preference in the existing preferences file as an optional boolean that defaults to disabled when missing or invalid. The preference SHALL be included in preferences reset behavior, restored on launch, and migrated from a legacy `preferences.conf` file that contains a `sync_scroll` line.

#### Scenario: Missing preference falls back to disabled
- **WHEN** the preferences file omits the `sync_scroll` setting
- **THEN** the editor starts with Sync scroll disabled

#### Scenario: Invalid value falls back to disabled
- **WHEN** the preferences file contains a `sync_scroll` value that is not a valid boolean
- **THEN** the editor starts with Sync scroll disabled rather than failing

#### Scenario: Preference round-trips
- **WHEN** the user enables Sync scroll and restarts the editor
- **THEN** Sync scroll remains enabled

#### Scenario: Reset restores disabled default
- **WHEN** the user resets preferences
- **THEN** Sync scroll is disabled

#### Scenario: Legacy preferences file migrates the setting
- **WHEN** the editor starts with a legacy `preferences.conf` containing `sync_scroll=true` and no `config.toml`
- **THEN** the value is migrated into `config.toml` and Sync scroll starts enabled
