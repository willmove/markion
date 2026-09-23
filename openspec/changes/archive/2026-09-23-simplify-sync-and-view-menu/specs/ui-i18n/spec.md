## MODIFIED Requirements

### Requirement: View mode UI chrome SHALL be localized
The system SHALL route all user-visible UI strings for the Source/Split Preview combined control and the distinct Visual Edit and Read controls through the i18n layer, including native menu items, in-app menu items, status feedback, and keyboard shortcut reference text. The top-level synchronization menu title SHALL use a dedicated localized “Sync” string so that descriptive Backup and Sync content outside that title remains unchanged.

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

#### Scenario: Synchronization menu title reflects the active language
- **WHEN** the active interface language changes
- **THEN** the native and in-app top-level synchronization menus render the localized equivalent of “Sync”
- **AND** Backup and Sync setup, status, settings, and sync-center content continue to use their existing localized wording
