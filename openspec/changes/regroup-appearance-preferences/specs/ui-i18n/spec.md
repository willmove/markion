## ADDED Requirements

### Requirement: Appearance preferences UI chrome SHALL be localized
Every user-visible string of the Preferences panel Appearance tab — the tab label and any section heading inside the tab — SHALL be routed through the i18n layer's `t` / `tf` functions and localized in every supported UI language.

#### Scenario: Appearance tab labels reflect the active language
- **WHEN** the active interface language is any of the supported languages and the user opens the Appearance tab
- **THEN** the tab label and appearance section chrome render in that language with no hard-coded English literals
