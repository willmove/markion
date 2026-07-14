## ADDED Requirements

### Requirement: Preferences panel SHALL expose Heading menu depth
The Preferences panel SHALL include a Heading menu depth control in its non-theme display settings with two choices: **H1–H5** (default) and **H1–H6**. Activating a choice SHALL apply the setting immediately, update Format menu contents, reinstall native menus, and persist the preference.

#### Scenario: Control appears in Preferences panel
- **WHEN** the Preferences panel is open
- **THEN** it shows the Heading menu depth control with the current choice highlighted

#### Scenario: Selecting H1–H6 applies immediately
- **WHEN** the user selects H1–H6
- **THEN** the Format menu includes H4–H6 on the next render and native menus are reinstalled

#### Scenario: Selecting H1–H5 applies immediately
- **WHEN** the user selects H1–H5 after using H1–H6
- **THEN** H6 disappears from the Format menu on the next render while H4 and H5 remain visible, and native menus are reinstalled

#### Scenario: Choice persists
- **WHEN** the user selects H1–H6 and restarts the editor
- **THEN** Heading menu depth remains H1–H6

### Requirement: Heading menu depth preference persistence
The editor SHALL persist Heading menu depth in `config.toml` as `heading_menu_max_level` with allowed values `5` or `6`. Missing or invalid values SHALL default to `5`. The preference SHALL be included in preferences reset behavior.

#### Scenario: Missing preference defaults to H1–H5
- **WHEN** `config.toml` omits `heading_menu_max_level`
- **THEN** the editor starts with Heading menu depth H1–H5

#### Scenario: Invalid value defaults to H1–H5
- **WHEN** `heading_menu_max_level` is present but not `5` or `6`
- **THEN** the editor treats the value as `5`

#### Scenario: Reset restores H1–H5
- **WHEN** the user resets preferences
- **THEN** Heading menu depth returns to H1–H5
