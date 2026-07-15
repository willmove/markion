# theme-preferences

## Purpose

Covers the in-app **Preferences panel** surface for choosing a theme (swatch grid) and an interface language (pill list), together with the persistence-format contract for those two choices. The built-in theme catalog itself (the 14 themes, ordering invariant, custom `.theme` loading) is described under `chrome-platform`; this capability focuses on the selection UI and the persisted-preferences contract.
## Requirements
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

### Requirement: Custom themes SHALL be authored as TOML files
User-authored (custom) themes SHALL be stored as `.toml` files in the themes directory, with a `name`, an `is_dark` flag, and a `[colors]` sub-table carrying the eight `ThemeColors` keys (`app_bg`, `panel_bg`, `surface_bg`, `text`, `muted`, `border`, `active_bg`, `active_text`). Color values SHALL be written as `"#rrggbb"` strings and SHALL deserialize leniently (a leading `#` is optional), so a hand-edited `app_bg = "10131a"` loads the same as `app_bg = "#10131a"`. Every color key SHALL be `#[serde(default)]` so a partial file loads with the fallback palette rather than failing. When the editor loads a custom theme and finds a legacy `.theme` (`key=value`) file of the same stem with no `.toml` beside it, it SHALL parse the legacy file, write out an equivalent `.toml`, leave the legacy `.theme` in place, and log the migration — the legacy file is then ignored on subsequent loads. Listing the themes directory SHALL dedupe by file stem so a migrated pair (`midnight.theme` + `midnight.toml`) surfaces as a single theme.

#### Scenario: A TOML custom theme round-trips
- **WHEN** a `midnight.toml` with `name = "Midnight"`, `is_dark = true`, and all eight `[colors]` keys is saved and reloaded
- **THEN** every color value is restored exactly, and the theme appears in `available_themes()` as a custom entry

#### Scenario: A partial TOML theme loads with the fallback palette
- **WHEN** a `.toml` custom theme omits some `[colors]` keys (e.g. only `app_bg` and `text` are present)
- **THEN** the missing keys take the default `ThemeColors` values and the file still loads

#### Scenario: A legacy `.theme` file migrates once to TOML
- **WHEN** the themes directory contains a `midnight.theme` (`key=value`) but no `midnight.toml`
- **THEN** the first load parses the legacy file, writes a `midnight.toml` next to it, and returns the migrated theme; the `midnight.theme` is left in place
- **AND** on the next load, the `midnight.toml` is read directly and the `midnight.theme` is not parsed again

#### Scenario: A migrated pair surfaces as a single theme
- **WHEN** the themes directory contains both `midnight.theme` and `midnight.toml`
- **THEN** `list_theme_definitions` returns exactly one `Midnight` entry, sourced from the `.toml`

### Requirement: Preferences panel SHALL expose Preview adaptive width
The Preferences panel SHALL include a Preview adaptive width toggle in its non-theme display settings. Activating the toggle SHALL apply the Read mode preview width behavior immediately and persist the preference. The panel SHALL additionally include a Sync scroll toggle in the same display settings; activating it SHALL apply the Split Preview proportional scroll-coupling behavior immediately and persist the preference.

#### Scenario: Toggle appears in Preferences panel
- **WHEN** the Preferences panel is open
- **THEN** it shows a Preview adaptive width toggle and a Sync scroll toggle, each reflecting its current enabled/disabled state

#### Scenario: Toggling Preview adaptive width applies immediately
- **WHEN** the user toggles Preview adaptive width
- **THEN** the active app state updates immediately
- **AND** Read mode preview layout reflects the new setting on the next render

#### Scenario: Toggling Preview adaptive width persists
- **WHEN** the user toggles Preview adaptive width
- **THEN** the preferences file is updated with the new boolean value

#### Scenario: Toggling Sync scroll applies immediately
- **WHEN** the user toggles Sync scroll in the Preferences panel
- **THEN** the active app state updates immediately
- **AND** the next Split Preview render couples or decouples the two panes' scroll positions accordingly

#### Scenario: Toggling Sync scroll persists
- **WHEN** the user toggles Sync scroll
- **THEN** the preferences file is updated with the new boolean value

### Requirement: Preferences panel SHALL expose supported display settings as controls
The Preferences panel SHALL expose focus mode, typewriter mode, code line numbers, Preview adaptive width, sidebar visibility, and sidebar tab as interactive controls when those preferences are already supported by the app state and preferences file. Activating a control SHALL apply the setting immediately and persist it through the existing preferences file.

#### Scenario: Boolean settings are editable in the panel
- **WHEN** the Preferences panel is open
- **THEN** focus mode, typewriter mode, code line numbers, Preview adaptive width, and sidebar visibility each render as an actionable control showing the current state

#### Scenario: Toggling a setting applies immediately
- **WHEN** the user activates a boolean Preferences control
- **THEN** the corresponding app state changes immediately
- **AND** the new value is persisted to the preferences file

#### Scenario: Sidebar tab is editable in the panel
- **WHEN** the Preferences panel is open
- **THEN** the sidebar tab preference renders as a mutually exclusive Files/Outline choice that indicates the current tab

#### Scenario: Selecting a sidebar tab applies immediately
- **WHEN** the user selects a different sidebar tab in the Preferences panel
- **THEN** the app switches the sidebar to that tab, keeps the sidebar visible, and persists the new tab

### Requirement: Preferences panel controls SHALL use editable affordances
The Preferences panel SHALL render configurable values with button-like or segmented-control affordances instead of plain `on` / `off` summary text. The controls SHALL use localized labels and active-theme colors.

#### Scenario: Boolean values are not plain text only
- **WHEN** the Preferences panel renders a boolean setting
- **THEN** the enabled and disabled states are presented as clickable button-like controls

#### Scenario: Controls follow active language and theme
- **WHEN** the active language or theme changes
- **THEN** Preferences panel control labels and colors update on the next render

### Requirement: Preferences panel SHALL show Language before Theme
The Preferences panel SHALL place the Language section before the Theme section so users can choose the UI language before reviewing localized theme and preference labels.

#### Scenario: Language section precedes Theme section
- **WHEN** the Preferences panel is open
- **THEN** the Language section appears above the Theme section

### Requirement: Preferences panel SHALL expose Heading menu depth
The Preferences panel SHALL include a Heading menu depth control in its non-theme display settings with two choices: **H1–H5** (default) and **H1–H6**. Activating a choice SHALL apply the setting immediately, update Format menu contents, reinstall native menus, and persist the preference.

#### Scenario: Control appears in Preferences panel
- **WHEN** the Preferences panel is open
- **THEN** it shows the Heading menu depth control with the current choice highlighted

#### Scenario: Selecting H1–H6 applies immediately
- **WHEN** the user selects H1–H6
- **THEN** the Format menu includes H4–H6 on the next render and native menus are reinstalled

#### Scenario: Selecting H1–H5 applies immediately
- **WHEN** the user selects H1–H5 after using H1–H6
- **THEN** H6 disappears from the Format menu on the next render while H4 and H5 remain visible, and native menus are reinstalled

#### Scenario: Choice persists
- **WHEN** the user selects H1–H6 and restarts the editor
- **THEN** Heading menu depth remains H1–H6

### Requirement: Heading menu depth preference persistence
The editor SHALL persist Heading menu depth in `config.toml` as `heading_menu_max_level` with allowed values `5` or `6`. Missing or invalid values SHALL default to `5`. The preference SHALL be included in preferences reset behavior.

#### Scenario: Missing preference defaults to H1–H5
- **WHEN** `config.toml` omits `heading_menu_max_level`
- **THEN** the editor starts with Heading menu depth H1–H5

#### Scenario: Invalid value defaults to H1–H5
- **WHEN** `heading_menu_max_level` is present but not `5` or `6`
- **THEN** the editor treats the value as `5`

#### Scenario: Reset restores H1–H5
- **WHEN** the user resets preferences
- **THEN** Heading menu depth returns to H1–H5

