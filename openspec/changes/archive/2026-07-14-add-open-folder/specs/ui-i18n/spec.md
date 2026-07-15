## ADDED Requirements

### Requirement: Open Folder UI chrome SHALL be localized
The system SHALL route every user-visible Open Folder string through the i18n layer, including the native and in-window File-menu labels, directory-picker prompt, and opening, success, cancellation, and failure status feedback.

#### Scenario: Open Folder menu label reflects the active language
- **WHEN** the active interface language changes
- **THEN** both the native File menu and the in-window File dropdown render Open Folder in the active language via `t(language, Msg::…)`

#### Scenario: Folder picker prompt reflects the active language
- **WHEN** the user invokes Open Folder
- **THEN** the directory-picker prompt is produced through the active language translation

#### Scenario: Folder selection feedback reflects the active language
- **WHEN** folder selection starts, succeeds, is canceled, or fails
- **THEN** the corresponding status text is produced through `t` or `tf` in the active language
