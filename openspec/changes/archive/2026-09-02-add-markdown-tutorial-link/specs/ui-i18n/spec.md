## ADDED Requirements

### Requirement: Markdown Reference tutorial link is localized

The system SHALL route the Markdown Reference overlay's Kenhuang tutorial-link label through the i18n `Msg` / `t` layer for every supported interface language. The destination URL SHALL be displayed verbatim rather than translated. Adding the label message SHALL require translations in every supported language or the build SHALL fail.

#### Scenario: Tutorial-link label follows the active language

- **WHEN** the Markdown Reference overlay is opened after the interface language changes
- **THEN** the tutorial-link label renders in the active language
- **AND** the visible URL is the language-selected Kenhuang destination displayed verbatim
