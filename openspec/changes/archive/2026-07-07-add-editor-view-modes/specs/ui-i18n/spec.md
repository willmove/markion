## ADDED Requirements

### Requirement: View mode UI chrome SHALL be localized
The system SHALL route all user-visible UI strings for Edit, Split Preview, and Read mode controls through the i18n layer, including native menu items, in-app menu items, status feedback, and keyboard shortcut reference text.

#### Scenario: View mode menu labels reflect the active language
- **WHEN** the active interface language changes
- **THEN** the native View menu and in-app View dropdown render the Edit, Split Preview, and Read mode entries in the active language

#### Scenario: View mode status feedback reflects the active language
- **WHEN** the user switches to Edit, Split Preview, or Read mode
- **THEN** the status bar message naming the active mode is produced through the active language translation

#### Scenario: Shortcut reference lists direct mode shortcuts
- **WHEN** the user opens the keyboard shortcut reference from the Help menu
- **THEN** the reference lists the direct shortcuts for Edit, Split Preview, and Read mode in the active language
