## ADDED Requirements

### Requirement: PDF viewing chrome and failures SHALL be localized
Every user-visible PDF viewing string SHALL flow through the application i18n
layer, including loading and provider-unavailable states,
encrypted/corrupt/oversized/page-limit/page-render failure categories, zoom and
fit-width controls, current-page and total-page labels, page-number validation,
document-action-unavailable feedback, and host-owned install/lifecycle errors.
The signed catalog SHALL provide plain-text PDF plugin identity and capability
labels in every supported locale. Missing host keys or incomplete signed
identity records SHALL fail validation before release.

#### Scenario: PDF controls follow the active language
- **WHEN** the interface language changes while a PDF tab is active
- **THEN** PDF toolbar labels, tooltips, page feedback, and available status text render through the active-language translations

#### Scenario: PDF failures follow the active language
- **WHEN** loading or rendering a PDF produces a user-visible failure category
- **THEN** the category and recovery guidance are localized without exposing untranslated native-library diagnostics

#### Scenario: PDF provider is missing
- **WHEN** a registered PDF is opened without an enabled compatible provider
- **THEN** the unavailable tab and plugin-manager action use localized host text and the signed localized provider identity
- **AND** no worker diagnostic or catalog-supplied rich markup is rendered

#### Scenario: Missing translation fails completeness validation
- **WHEN** a PDF message key or signed PDF identity record lacks a value in any supported language
- **THEN** localization completeness validation fails before the change can be released
