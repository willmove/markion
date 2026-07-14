## ADDED Requirements

### Requirement: Heading menu depth UI chrome SHALL be localized
The system SHALL route every user-visible string for Heading menu depth and H4–H6 Format menu entries through the i18n layer, including menu item labels, Preferences panel control labels, and keyboard shortcut reference entries when H1–H6 depth is active.

#### Scenario: H4–H6 menu labels reflect active language
- **WHEN** Heading menu depth is H1–H6 and the active interface language is Simplified Chinese
- **THEN** H4, H5, and H6 Format menu items render in Simplified Chinese

#### Scenario: Preferences panel label reflects active language
- **WHEN** the active interface language changes
- **THEN** the Heading menu depth control label and option text render in the active language
