## REMOVED Requirements

### Requirement: View mode UI chrome SHALL be localized
**Reason**: The requirement text and one scenario referenced the dedicated top-level "Sync" menu title, which no longer exists; the localization obligation itself continues unchanged in the replacement requirement below.
**Migration**: See "View mode UI chrome SHALL be localized through the i18n layer" for the same obligation without the retired top-level menu title.

## ADDED Requirements

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
- **WHEN** the user opens the keyboard shortcut reference from the Help menu
- **THEN** the reference lists one localized Source/Split Preview action with `Ctrl+/` on Windows and Linux or `Cmd+/` on macOS
- **AND** it does not list a separate Split Preview shortcut
