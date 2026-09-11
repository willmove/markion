## ADDED Requirements

### Requirement: PDF viewing chrome and failures SHALL be localized
Every user-visible PDF viewing string SHALL flow through the application i18n layer, including loading and unavailable states, encrypted/corrupt/oversized/page-limit/page-render failure categories, zoom and fit-width controls, current-page and total-page labels, page-number validation, and document-action-unavailable feedback. Adding these message keys SHALL require translations in every supported language or localization completeness tests SHALL fail.

#### Scenario: PDF controls follow the active language
- **WHEN** the interface language changes while a PDF tab is active
- **THEN** PDF toolbar labels, tooltips, page feedback, and available status text render through the active-language translations

#### Scenario: PDF failures follow the active language
- **WHEN** loading or rendering a PDF produces a user-visible failure category
- **THEN** the category and recovery guidance are localized without exposing untranslated native-library diagnostics

#### Scenario: Missing translation fails completeness validation
- **WHEN** a PDF message key lacks a value in any supported language
- **THEN** localization completeness validation fails before the change can be released
