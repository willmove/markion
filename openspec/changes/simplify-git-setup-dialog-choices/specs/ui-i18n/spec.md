## ADDED Requirements

### Requirement: Setup confirm and cancel chrome SHALL be localized
The generic setup confirm and cancel button labels SHALL be translated through the i18n layer in every supported language. The removed route-disclosure labels ("other setup options" / "hide other setup options") SHALL no longer appear in the i18n surface.

#### Scenario: Confirm and cancel follow the UI language
- **WHEN** the setup dialog is shown
- **THEN** its bottom confirm and cancel buttons appear in the active interface language

#### Scenario: Disclosure labels are gone
- **WHEN** the setup dialog is shown in any language
- **THEN** no "other setup options" or "hide other setup options" string is rendered
