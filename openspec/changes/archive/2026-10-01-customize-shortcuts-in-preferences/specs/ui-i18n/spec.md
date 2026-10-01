# ui-i18n

## MODIFIED Requirements

### Requirement: View mode UI chrome SHALL be localized through the i18n layer
The system SHALL route all user-visible UI strings for the Source/Split Preview combined control and the distinct Visual Edit and Read controls through the i18n layer, including native menu items, in-app menu items, status feedback, and keyboard shortcut reference text.

#### Scenario: View mode menu labels reflect the active language
- **WHEN** the active interface language changes
- **THEN** the native View menu and in-app View dropdown render one Source/Split Preview entry plus the Visual Edit and Read entries in the active language
- **AND** separate Source and Split Preview entries are not rendered

#### Scenario: View mode status feedback reflects the active language
- **WHEN** the user switches to Source, Split Preview, Visual Edit, or Read mode
- **THEN** the status bar message naming the active mode is produced through the active language translation

#### Scenario: Shortcut reference lists direct mode shortcuts
- **WHEN** the user opens the Shortcuts tab of the Preferences panel
- **THEN** the reference lists one localized Source/Split Preview action with `Ctrl+/` on Windows and Linux or `Cmd+/` on macOS
- **AND** it does not list a separate Split Preview shortcut

## ADDED Requirements

### Requirement: Shortcut customization UI chrome SHALL be localized
The system SHALL route every user-visible string of the Preferences panel Shortcuts tab through the i18n layer, including the tab labels, the capture prompt, the conflict and invalid-keystroke feedback, and the per-action reset affordance, for every supported interface language.

#### Scenario: Shortcuts tab labels reflect the active language
- **WHEN** the active interface language changes
- **THEN** the Shortcuts tab label, capture prompt, and reset control render in the active language

#### Scenario: Conflict feedback reflects the active language
- **WHEN** a shortcut assignment is rejected for conflict or invalid input
- **THEN** the inline feedback message is produced through the active language translation
