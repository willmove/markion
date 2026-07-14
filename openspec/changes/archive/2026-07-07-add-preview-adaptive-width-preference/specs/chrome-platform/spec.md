## ADDED Requirements

### Requirement: Read mode preview width
In Read mode, the editor SHALL constrain rendered preview content to a default maximum width of 860px and center that content within the available preview pane. The editor SHALL provide a persisted "Preview adaptive width" preference that is disabled by default; when enabled, Read mode rendered preview content SHALL use the full available preview width. This width preference SHALL NOT affect Edit mode or Split Preview mode.

#### Scenario: Read mode defaults to readable width
- **WHEN** the active view mode is Read and Preview adaptive width is disabled
- **THEN** rendered preview content is centered and constrained to a maximum width of 860px

#### Scenario: Adaptive width restores full-width Read mode
- **WHEN** the active view mode is Read and Preview adaptive width is enabled
- **THEN** rendered preview content uses the full available preview pane width

#### Scenario: Split Preview mode remains full pane width
- **WHEN** the active view mode is Split Preview
- **THEN** rendered preview content uses the full preview pane width regardless of the Preview adaptive width preference

### Requirement: Preview adaptive width preference persistence
The editor SHALL persist the Preview adaptive width preference in the existing preferences file as an optional boolean that defaults to disabled when missing or invalid. The preference SHALL be included in preferences reset behavior and restored on launch.

#### Scenario: Missing preference falls back to disabled
- **WHEN** the preferences file omits the Preview adaptive width setting
- **THEN** the editor starts with Preview adaptive width disabled

#### Scenario: Preference round-trips
- **WHEN** the user enables Preview adaptive width and restarts the editor
- **THEN** Preview adaptive width remains enabled

#### Scenario: Reset restores readable default
- **WHEN** the user resets preferences
- **THEN** Preview adaptive width is disabled
