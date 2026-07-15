## ADDED Requirements

### Requirement: Preview context menu strings are localized
The system SHALL route all user-visible labels and status feedback for the preview pane context menu (Copy as Plain Text, Copy as Markdown, Copy as HTML, Select All, Copy Link Address, and related status messages) through the i18n `Msg` / `t` / `tf` layer for every supported interface language.

#### Scenario: Preview context menu labels follow the active language
- **WHEN** the active interface language is Simplified Chinese and the preview context menu is open
- **THEN** every menu item label is produced via `t(language, Msg::…)` in Simplified Chinese

#### Scenario: Preview copy status feedback is localized
- **WHEN** the user copies preview content as plain text, Markdown, or HTML from the context menu
- **THEN** the status bar message is produced through the active language translation
