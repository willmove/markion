## ADDED Requirements

### Requirement: The editor SHALL ship a fixed catalog of built-in themes
The system SHALL provide a built-in theme catalog of at least fourteen themes returned by `builtin_theme_definitions()`, covering the original six (Paper, Ink, Solar, Forest, Rose, Graphite) plus popular editor palettes (GitHub Light/Dark, Solarized Light/Dark, One Light/Dark, Tokyo Night/Light). Each theme SHALL carry a stable name, a dark/light flag, and a `ThemeColors` palette. Theme names are identity keys written to the preferences file, so renames SHALL be avoided to prevent orphaning saved selections.

#### Scenario: Built-in catalog includes the legacy themes first
- **WHEN** `builtin_theme_definitions()` is called
- **THEN** the first six entries are Paper, Ink, Solar, Forest, Rose, Graphite in that exact order, so the legacy `cycle_theme` path and its test continue to hold

#### Scenario: Built-in catalog includes popular editor palettes
- **WHEN** the preferences panel enumerates available themes
- **THEN** the list includes GitHub Light, GitHub Dark, Solarized Light, Solarized Dark, One Light, One Dark, Tokyo Night, and Tokyo Night Light, each with a unique name

### Requirement: The Preferences panel SHALL let the user choose a theme by swatch
The system SHALL render a Preferences panel containing a swatch grid where each theme (built-in plus any custom `.theme` files) is a card showing a preview of representative palette colors, the theme name, and a check mark on the active theme. Activating a card SHALL apply that theme immediately and persist the choice.

#### Scenario: Theme cards show a color preview and the active marker
- **WHEN** the Preferences panel is open
- **THEN** each theme card displays a multi-segment color swatch drawn from the theme palette and shows a check mark only on the currently active theme

#### Scenario: Selecting a theme applies and persists it
- **WHEN** the user clicks a theme card
- **THEN** that theme becomes active immediately, the preferences file is updated with its name, and the active card receives a highlighted border

#### Scenario: Custom themes appear alongside built-ins
- **WHEN** custom `.theme` files exist in the themes directory
- **THEN** they appear in the swatch grid together with the built-in themes, with built-ins winning on name collisions

### Requirement: Theme and language choices SHALL persist in a single preferences file
The system SHALL persist the selected theme and the selected interface language as lines in the same preferences file (`theme=<name>`, `language=<code>`). Reading the preferences file SHALL tolerate the absence of either line by applying the documented default, and SHALL tolerate unknown values by falling back to the default.

#### Scenario: Language and theme round-trip through the preferences file
- **WHEN** preferences with a theme of `Forest` and a language of `zh` are saved and reloaded
- **THEN** both values are restored exactly

#### Scenario: Older preferences files without a language line upgrade gracefully
- **WHEN** a preferences file written by an older build (no `language=` line) is loaded
- **THEN** the language defaults to English and the theme is honored as before
