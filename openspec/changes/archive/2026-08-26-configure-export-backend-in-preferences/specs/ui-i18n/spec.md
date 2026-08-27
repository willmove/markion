## ADDED Requirements

### Requirement: Export preferences UI chrome SHALL be localized
Every user-visible string of the Preferences panel Export tab — the tab label, backend choice labels, availability status lines, pandoc path / reference template rows and their actions, PDF-engine choices, and every format option label — SHALL be routed through the i18n layer's `t` / `tf` functions and localized in every supported UI language.

#### Scenario: Export tab labels reflect the active language
- **WHEN** the active interface language is any of the supported languages and the user opens the Export tab
- **THEN** every label, action, and status line in the tab renders in that language with no hard-coded English literals
