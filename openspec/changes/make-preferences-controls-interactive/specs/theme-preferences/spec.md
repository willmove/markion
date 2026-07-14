## ADDED Requirements

### Requirement: Preferences panel SHALL expose supported display settings as controls
The Preferences panel SHALL expose focus mode, typewriter mode, code line numbers, Preview adaptive width, sidebar visibility, and sidebar tab as interactive controls when those preferences are already supported by the app state and preferences file. Activating a control SHALL apply the setting immediately and persist it through the existing preferences file.

#### Scenario: Boolean settings are editable in the panel
- **WHEN** the Preferences panel is open
- **THEN** focus mode, typewriter mode, code line numbers, Preview adaptive width, and sidebar visibility each render as an actionable control showing the current state

#### Scenario: Toggling a setting applies immediately
- **WHEN** the user activates a boolean Preferences control
- **THEN** the corresponding app state changes immediately
- **AND** the new value is persisted to the preferences file

#### Scenario: Sidebar tab is editable in the panel
- **WHEN** the Preferences panel is open
- **THEN** the sidebar tab preference renders as a mutually exclusive Files/Outline choice that indicates the current tab

#### Scenario: Selecting a sidebar tab applies immediately
- **WHEN** the user selects a different sidebar tab in the Preferences panel
- **THEN** the app switches the sidebar to that tab, keeps the sidebar visible, and persists the new tab

### Requirement: Preferences panel controls SHALL use editable affordances
The Preferences panel SHALL render configurable values with button-like or segmented-control affordances instead of plain `on` / `off` summary text. The controls SHALL use localized labels and active-theme colors.

#### Scenario: Boolean values are not plain text only
- **WHEN** the Preferences panel renders a boolean setting
- **THEN** the enabled and disabled states are presented as clickable button-like controls

#### Scenario: Controls follow active language and theme
- **WHEN** the active language or theme changes
- **THEN** Preferences panel control labels and colors update on the next render

### Requirement: Preferences panel SHALL show Language before Theme
The Preferences panel SHALL place the Language section before the Theme section so users can choose the UI language before reviewing localized theme and preference labels.

#### Scenario: Language section precedes Theme section
- **WHEN** the Preferences panel is open
- **THEN** the Language section appears above the Theme section
