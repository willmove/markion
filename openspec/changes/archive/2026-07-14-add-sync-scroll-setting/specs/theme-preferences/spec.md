## MODIFIED Requirements

### Requirement: Preferences panel SHALL expose Preview adaptive width
The Preferences panel SHALL include a Preview adaptive width toggle in its non-theme display settings. Activating the toggle SHALL apply the Read mode preview width behavior immediately and persist the preference. The panel SHALL additionally include a Sync scroll toggle in the same display settings; activating it SHALL apply the Split Preview proportional scroll-coupling behavior immediately and persist the preference.

#### Scenario: Toggle appears in Preferences panel
- **WHEN** the Preferences panel is open
- **THEN** it shows a Preview adaptive width toggle and a Sync scroll toggle, each reflecting its current enabled/disabled state

#### Scenario: Toggling Preview adaptive width applies immediately
- **WHEN** the user toggles Preview adaptive width
- **THEN** the active app state updates immediately
- **AND** Read mode preview layout reflects the new setting on the next render

#### Scenario: Toggling Preview adaptive width persists
- **WHEN** the user toggles Preview adaptive width
- **THEN** the preferences file is updated with the new boolean value

#### Scenario: Toggling Sync scroll applies immediately
- **WHEN** the user toggles Sync scroll in the Preferences panel
- **THEN** the active app state updates immediately
- **AND** the next Split Preview render couples or decouples the two panes' scroll positions accordingly

#### Scenario: Toggling Sync scroll persists
- **WHEN** the user toggles Sync scroll
- **THEN** the preferences file is updated with the new boolean value
