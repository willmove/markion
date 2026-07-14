## ADDED Requirements

### Requirement: File tree metadata action UI chrome SHALL be localized
The system SHALL route all user-visible strings for the file-tree Copy Path, Copy Relative Path, and Properties actions through the i18n layer, including context-menu labels, properties field labels, success status messages, and failure status messages.

#### Scenario: Metadata action labels reflect the active language
- **WHEN** the active interface language changes
- **THEN** the file-tree context menu renders Copy Path, Copy Relative Path, and Properties in the active language

#### Scenario: Properties dialog text reflects the active language
- **WHEN** the user opens file-tree Properties
- **THEN** the properties title, field labels, unavailable values, and close button text are produced through the active language translation

#### Scenario: Metadata action status feedback reflects the active language
- **WHEN** the editor reports success or failure for Copy Path, Copy Relative Path, or Properties
- **THEN** the status text is produced through the active language translation
