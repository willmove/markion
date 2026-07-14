## ADDED Requirements

### Requirement: Preview adaptive width UI chrome SHALL be localized
The system SHALL route every user-visible string for the Preview adaptive width preference through the i18n layer, including the Preferences panel label, preferences summary text, and any related status feedback.

#### Scenario: Preferences panel label reflects active language
- **WHEN** the active interface language changes
- **THEN** the Preview adaptive width label in the Preferences panel renders in the active language

#### Scenario: Preferences summary reflects active language
- **WHEN** the user opens preferences summary text
- **THEN** the Preview adaptive width value is displayed through localized text
