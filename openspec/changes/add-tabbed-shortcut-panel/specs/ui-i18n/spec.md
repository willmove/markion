## ADDED Requirements

### Requirement: Keyboard shortcut panel content is localized and platform explicit
The system SHALL source the keyboard shortcut panel title, category labels, action labels, controls, and status feedback from the i18n layer for every supported interface language. The shortcut catalog SHALL provide distinct Windows/Linux and macOS shortcut combinations using user-facing key names such as Ctrl, Cmd, Alt, Option, and Shift, and SHALL NOT expose GPUI-internal names such as `Secondary` or table-formatting source text.

#### Scenario: Shortcut panel follows the active language
- **WHEN** the user opens Help -> Keyboard Shortcuts after selecting any supported interface language
- **THEN** the panel title, category labels, action labels, controls, and status feedback render in that language

#### Scenario: Platform tabs use explicit key names
- **WHEN** the user switches between Windows/Linux and macOS in the shortcut panel
- **THEN** each action displays the shortcut combinations for the selected platform using explicit user-facing modifier names
- **AND** no `Secondary-*`, Markdown table syntax, or ASCII table rules appear

#### Scenario: Heading depth is reflected in the catalog
- **WHEN** Heading menu depth is configured for H1-H6 and the shortcut panel is opened
- **THEN** the localized heading action and shortcut combinations include H6 consistently with the Format menu
