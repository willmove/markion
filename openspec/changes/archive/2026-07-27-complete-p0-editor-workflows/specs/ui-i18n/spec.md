## ADDED Requirements

### Requirement: P0 authoring and conflict workflows SHALL be localized
Every user-visible label, status, prompt, validation error, missing-resource message, contextual-control tooltip, link/image editor field, and external-change/recovery action introduced by this change SHALL be routed through the localization catalog for every supported language.

#### Scenario: P0 workflows render in each supported language
- **WHEN** the application language changes while a resource, link-editor, missing-resource, external-change, or recovery surface is visible
- **THEN** all user-facing text on that surface uses the selected language
- **AND** no implementation-only English literal is presented as UI text
