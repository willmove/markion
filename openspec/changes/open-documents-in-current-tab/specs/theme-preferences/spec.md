## ADDED Requirements

### Requirement: Preferences panel SHALL expose an Open documents in current tab toggle

The Preferences panel SHALL include an Open-documents-in-current-tab toggle in its non-theme display settings, reflecting the current enabled/disabled state. Activating the toggle SHALL apply the new default open-target behavior to subsequent non-explicit open actions immediately (no restart required) and SHALL persist the preference. The toggle SHALL default to on.

#### Scenario: Toggle appears in the Preferences panel
- **WHEN** the Preferences panel is open
- **THEN** it shows an Open-documents-in-current-tab toggle in the display-settings area, rendered as an actionable control reflecting the current state

#### Scenario: Toggling applies to the next open action
- **WHEN** the user turns the toggle off and then clicks a different supported file in the file tree with a clean active tab
- **THEN** that file opens in a new appended tab instead of replacing the current tab
- **AND** turning the toggle back on restores replace-on-open for subsequent clicks

#### Scenario: Toggling persists
- **WHEN** the user toggles Open documents in current tab
- **THEN** the preferences file is updated with the new boolean value

### Requirement: Open-in-current-tab preference SHALL persist safely

The editor SHALL persist the Open-in-current-tab preference in `config.toml` as a boolean, defaulting to on. A missing field SHALL default to on, and a non-boolean field SHALL degrade to the default (on) rather than preventing the editor from starting. The preference SHALL be included in preferences reset behavior.

#### Scenario: Missing preference defaults to on
- **WHEN** `config.toml` omits the Open-in-current-tab field
- **THEN** the editor starts with open-in-current-tab behavior on

#### Scenario: Invalid value defaults to on
- **WHEN** the Open-in-current-tab field is present but not a valid boolean
- **THEN** the editor treats the value as on and the preferences file does not prevent the editor from starting

#### Scenario: Value round-trips
- **WHEN** preferences with Open-in-current-tab disabled are saved and reloaded
- **THEN** the disabled state is restored exactly and reflected by the Preferences control

#### Scenario: Reset restores on
- **WHEN** the user resets preferences after disabling Open-in-current-tab
- **THEN** the preference returns to its default on state
