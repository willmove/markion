## ADDED Requirements

### Requirement: Theme preferences UI chrome SHALL be localized
Every user-visible string of the Preferences panel Theme tab — the tab label and any section heading inside the tab — SHALL be routed through the i18n layer's `t` / `tf` functions and localized in every supported UI language.

#### Scenario: Theme tab labels reflect the active language
- **WHEN** the active interface language is any of the supported languages and the user opens the Theme tab
- **THEN** the tab label and theme section chrome render in that language with no hard-coded English literals
