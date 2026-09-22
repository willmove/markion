## ADDED Requirements

### Requirement: Sync adoption and message-template chrome SHALL be localized
The automatic-adoption status notification and the message-template placeholder help SHALL be user-visible chrome translated through the i18n layer in every supported language. The automatically generated commit message text itself is repository content produced by the sync engine, not localized chrome, and SHALL NOT be translated per UI language.

#### Scenario: Adoption notification follows the UI language
- **WHEN** an existing repository is adopted for sync
- **THEN** the status notification reporting the connected repository appears in the active interface language

#### Scenario: Template help lists placeholders in the UI language
- **WHEN** settings display help for the sync message template
- **THEN** the available placeholders, including date, time, files, and count, are explained in the active interface language
