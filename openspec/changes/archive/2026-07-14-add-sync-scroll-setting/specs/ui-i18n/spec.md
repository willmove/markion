## ADDED Requirements

### Requirement: Sync scroll UI chrome SHALL be localized
The system SHALL route every user-visible string for the Sync scroll preference through the i18n layer, including the Preferences panel label, the on/off status feedback when the preference is toggled, and any related preferences summary text. Adding the new message keys SHALL require translations in every supported language or the build SHALL fail.

#### Scenario: Preferences panel label reflects active language
- **WHEN** the active interface language changes
- **THEN** the Sync scroll label in the Preferences panel renders in the active language

#### Scenario: Toggle status feedback reflects active language
- **WHEN** the user toggles Sync scroll
- **THEN** the status bar message indicating Sync scroll on or off is produced through the active language translation

#### Scenario: New message keys require all-language translations
- **WHEN** a developer adds the Sync scroll message variants to `Msg`
- **THEN** the project fails to compile until both English and Simplified Chinese cover them via the exhaustive `match`
