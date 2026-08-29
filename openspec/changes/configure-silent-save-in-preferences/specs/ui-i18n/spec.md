## ADDED Requirements

### Requirement: Silent-save preference strings SHALL be localized
Every user-visible string for the Preferences Auto-save section (section title if any, silent save-to-file control label, auto-save delay label, and any on/off or validation status feedback for those controls) SHALL be routed through the i18n layer and provided for every supported UI language. Adding the new message keys SHALL require translations in every supported language or the build SHALL fail. Recovery-only autosave status for named documents SHALL continue to use localized recovery-saved messaging rather than destination auto-save messaging.

#### Scenario: Preferences panel labels reflect active language
- **WHEN** the interface language changes while the Preferences Auto-save controls are visible
- **THEN** the silent-save and delay labels render in the active language

#### Scenario: Catalog completeness is enforced
- **WHEN** a new Auto-save preference message key is added
- **THEN** every supported language provides a translation or the build fails
