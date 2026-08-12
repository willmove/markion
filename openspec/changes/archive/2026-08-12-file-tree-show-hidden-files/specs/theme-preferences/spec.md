## ADDED Requirements

### Requirement: Preferences panel SHALL expose a Show hidden files toggle

The Preferences panel SHALL include a Show hidden files/folders toggle in its non-theme display settings, reflecting the current enabled/disabled state. Activating the toggle SHALL apply the new file-tree hidden-entry visibility behavior immediately by re-scanning the workspace, and SHALL persist the preference. The toggle SHALL default to off.

#### Scenario: Toggle appears in the Preferences panel
- **WHEN** the Preferences panel is open
- **THEN** it shows a Show hidden files/folders toggle in the display-settings area, rendered as an actionable control reflecting the current state

#### Scenario: Toggling applies immediately and re-scans the tree
- **WHEN** the user activates the Show hidden files/folders toggle
- **THEN** the workspace is re-scanned under the new visibility rule
- **AND** the file tree updates to reveal or hide hidden entries on the next render

#### Scenario: Toggling persists
- **WHEN** the user toggles Show hidden files/folders
- **THEN** the preferences file is updated with the new boolean value

### Requirement: Show hidden files preference SHALL persist safely

The editor SHALL persist the Show-hidden-files preference in `config.toml` as a boolean, defaulting to off. A missing field SHALL default to off, and a non-boolean field SHALL degrade to off rather than preventing the editor from starting. The preference SHALL be included in preferences reset behavior.

#### Scenario: Missing preference defaults to off
- **WHEN** `config.toml` omits the Show-hidden-files field
- **THEN** the editor starts with hidden-entry visibility off

#### Scenario: Invalid value defaults to off
- **WHEN** the Show-hidden-files field is present but not a valid boolean
- **THEN** the editor treats the value as off and the preferences file does not prevent the editor from starting

#### Scenario: Value round-trips
- **WHEN** preferences with Show-hidden-files enabled are saved and reloaded
- **THEN** the enabled state is restored exactly and reflected by the Preferences control

#### Scenario: Reset restores off
- **WHEN** the user resets preferences after enabling Show-hidden-files
- **THEN** the preference returns to off and hidden entries are omitted from the tree on the next scan
