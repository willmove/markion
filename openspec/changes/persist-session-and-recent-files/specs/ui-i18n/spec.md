## ADDED Requirements

### Requirement: Open Recent UI chrome SHALL be localized
The system SHALL route every user-visible Open Recent string through the i18n layer, including the in-window File-menu Open Recent label, the empty recent-list placeholder, the Clear Recent Files action label, and any status feedback for opening a recent path or clearing the list. Adding the new message keys SHALL require translations in every supported language or the build SHALL fail.

#### Scenario: Open Recent menu labels reflect the active language
- **WHEN** the active interface language changes
- **THEN** the in-window File → Open Recent label, empty-state placeholder, and Clear Recent Files label render via `t(language, Msg::…)` in the active language

#### Scenario: Recent-file open and clear feedback is localized
- **WHEN** the user opens a recent file successfully or unsuccessfully, or clears the recent-files list
- **THEN** the corresponding status text is produced through `t` or `tf` in the active language

#### Scenario: New Open Recent message keys require all-language translations
- **WHEN** a developer adds Open Recent message variants to `Msg`
- **THEN** the project fails to compile until every supported language covers them via the exhaustive `match`
