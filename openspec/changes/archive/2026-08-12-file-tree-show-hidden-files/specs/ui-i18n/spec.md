## ADDED Requirements

### Requirement: Show hidden files UI chrome SHALL be localized

The system SHALL route every user-visible string for the Show-hidden-files preference through the i18n layer, including the Preferences panel toggle label and any on/off status feedback reported when the preference is toggled. Adding the new message keys SHALL require translations in every supported language or the build SHALL fail.

#### Scenario: Preferences panel label reflects active language
- **WHEN** the active interface language changes
- **THEN** the Show-hidden-files toggle label in the Preferences panel renders in the active language

#### Scenario: Toggle status feedback reflects active language
- **WHEN** the user toggles Show-hidden-files
- **THEN** any status bar message indicating the new on/off state is produced through the active language translation

#### Scenario: New message keys require all-language translations
- **WHEN** a developer adds the Show-hidden-files message variants to `Msg`
- **THEN** the project fails to compile until every supported language covers them via the exhaustive `match`
