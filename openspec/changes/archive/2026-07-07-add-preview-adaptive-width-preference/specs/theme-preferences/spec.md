## ADDED Requirements

### Requirement: Preferences panel SHALL expose Preview adaptive width
The Preferences panel SHALL include a Preview adaptive width toggle in its non-theme display settings. Activating the toggle SHALL apply the Read mode preview width behavior immediately and persist the preference.

#### Scenario: Toggle appears in Preferences panel
- **WHEN** the Preferences panel is open
- **THEN** it shows a Preview adaptive width toggle with the current enabled/disabled state

#### Scenario: Toggling applies immediately
- **WHEN** the user toggles Preview adaptive width
- **THEN** the active app state updates immediately
- **AND** Read mode preview layout reflects the new setting on the next render

#### Scenario: Toggling persists
- **WHEN** the user toggles Preview adaptive width
- **THEN** the preferences file is updated with the new boolean value
