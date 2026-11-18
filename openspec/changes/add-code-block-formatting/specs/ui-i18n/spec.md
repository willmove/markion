## ADDED Requirements

### Requirement: Code block formatting UI chrome SHALL be localized
Every user-visible label, button caption, tooltip, status feedback message, Preferences -> Editor -> Code Blocks setting description, and formatter availability string introduced for code block formatting SHALL be translated through the localized message catalog across all eight supported interface languages (`En`, `ZhHans`, `ZhHant`, `Ja`, `Fr`, `De`, `Es`, `Ko`) without hard-coded English literals. Changing the interface language SHALL relabel visible code-block formatting controls without mutating document content or caches.

#### Scenario: Code block format chrome follows the active interface language
- **WHEN** the interface language switches among any of the eight supported languages while Preferences -> Editor -> Code Blocks or code-block format controls are visible
- **THEN** the Format Code Block labels, Editor tab controls, availability labels, and status messages render in the active language
- **AND** document text, version, and undo history remain unchanged
