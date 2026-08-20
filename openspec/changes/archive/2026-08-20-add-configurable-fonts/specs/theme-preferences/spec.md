## MODIFIED Requirements

### Requirement: Custom themes SHALL be authored as TOML files

User-authored (custom) themes SHALL be stored as `.toml` files in the themes directory, with a `name`, an `is_dark` flag, and a `[colors]` sub-table carrying the eight `ThemeColors` keys (`app_bg`, `panel_bg`, `surface_bg`, `text`, `muted`, `border`, `active_bg`, `active_text`). Color values SHALL be written as `"#rrggbb"` strings and SHALL deserialize leniently (a leading `#` is optional), so a hand-edited `app_bg = "10131a"` loads the same as `app_bg = "#10131a"`. Every color key SHALL be `#[serde(default)]` so a partial file loads with the fallback palette rather than failing. A theme MAY additionally carry a `[fonts]` sub-table with optional string keys `editor`, `rendered`, and `code`, each naming a font family that the theme contributes to the corresponding document plane slot when the user has no explicit preference for it; absent keys and an absent `[fonts]` table SHALL mean the theme specifies no font for that slot, and an empty or whitespace-only value SHALL be treated as absent. When the editor loads a custom theme and finds a legacy `.theme` (`key=value`) file of the same stem with no `.toml` beside it, it SHALL parse the legacy file, write out an equivalent `.toml`, leave the legacy `.theme` in place, and log the migration — the legacy file is then ignored on subsequent loads. Listing the themes directory SHALL dedupe by file stem so a migrated pair (`midnight.theme` + `midnight.toml`) surfaces as a single theme.

#### Scenario: A TOML custom theme round-trips

- **WHEN** a `midnight.toml` with `name = "Midnight"`, `is_dark = true`, and all eight `[colors]` keys is saved and reloaded
- **THEN** every color value is restored exactly, and the theme appears in `available_themes()` as a custom entry

#### Scenario: A partial TOML theme loads with the fallback palette

- **WHEN** a `.toml` custom theme omits some `[colors]` keys (e.g. only `app_bg` and `text` are present)
- **THEN** the missing keys take the default `ThemeColors` values and the file still loads

#### Scenario: Theme fonts round-trip and apply per slot

- **WHEN** a `.toml` custom theme carries `[fonts]` with `rendered = "Georgia"` and the user has no explicit rendered-font preference
- **THEN** the theme loads with that font recorded for the rendered slot, the value survives a save/reload round-trip, and rendered body text uses Georgia while that theme is active

#### Scenario: A theme without fonts or with partial fonts loads unchanged

- **WHEN** a `.toml` custom theme has no `[fonts]` table, or a `[fonts]` table listing only some keys
- **THEN** the theme loads exactly as before this change and unspecified slots fall back to their defaults (no theme contribution)

#### Scenario: Empty font values are treated as absent

- **WHEN** a `[fonts]` key is present but empty or whitespace-only
- **THEN** the theme loads successfully and that slot behaves as if the key were absent

#### Scenario: First use installs a sample theme with fonts

- **WHEN** the themes directory does not exist and the Preferences panel first opens
- **THEN** the editor creates the directory and writes a sample `typewriter.toml` (light palette) whose `[fonts]` table demonstrates the optional editor/rendered/code contributions
- **AND** the sample loads as a selectable custom theme alongside the built-ins; an existing themes directory is never modified

#### Scenario: A legacy `.theme` file migrates once to TOML

- **WHEN** the themes directory contains a `midnight.theme` (`key=value`) but no `midnight.toml`
- **THEN** the first load parses the legacy file, writes a `midnight.toml` next to it, and returns the migrated theme; the `midnight.theme` is left in place
- **AND** on the next load, the `midnight.toml` is read directly and the `midnight.theme` is not parsed again

#### Scenario: A migrated pair surfaces as a single theme

- **WHEN** the themes directory contains both `midnight.theme` and `midnight.toml`
- **THEN** `list_theme_definitions` returns exactly one `Midnight` entry, sourced from the `.toml`

### Requirement: Document typography preferences SHALL persist safely

The editor SHALL persist source font size as `editor_font_size`, rendered font size as `rendered_font_size`, and rendered paragraph spacing as `paragraph_spacing` in `config.toml`, with defaults of 14px, 14px, and 12px respectively. Font sizes SHALL normalize to 10–32px inclusive and paragraph spacing SHALL normalize to 0–32px inclusive. The editor SHALL additionally persist optional font-family preferences as string keys `editor_font_family`, `rendered_font_family`, and `code_font_family` in `config.toml`; each key SHALL be optional, an absent, empty, or whitespace-only value SHALL mean "follow the active theme, then the built-in default", and a present value SHALL be persisted and reloaded verbatim without validating it against the set of installed fonts. Missing or non-numeric size/spacing fields SHALL use their defaults, numeric out-of-range fields SHALL clamp to the nearest bound, and reset SHALL restore all three size/spacing defaults and clear all three font-family preferences to the follow-theme state.

#### Scenario: Typography values round-trip

- **WHEN** preferences containing `editor_font_size = 18`, `rendered_font_size = 20`, and `paragraph_spacing = 16` are saved and reloaded
- **THEN** all three values are restored exactly and reflected by the Preferences controls

#### Scenario: Older config uses current defaults

- **WHEN** an existing `config.toml` omits all typography fields
- **THEN** the editor starts with 14px source text, 14px rendered body text, and 12px rendered paragraph spacing

#### Scenario: Invalid and out-of-range values are safe

- **WHEN** typography fields are non-numeric or outside their supported ranges
- **THEN** non-numeric values use defaults and numeric values clamp to their nearest supported bound
- **AND** the preferences file does not prevent the editor from starting

#### Scenario: Font-family preferences round-trip verbatim

- **WHEN** `config.toml` contains `editor_font_family = "Cascadia Code"` and `rendered_font_family = "Source Serif 4"` and preferences are saved and reloaded
- **THEN** both values are restored exactly and apply to their document planes

#### Scenario: Absent font-family keys follow the theme

- **WHEN** `config.toml` omits all three font-family keys or contains an empty value for one
- **THEN** the corresponding slot resolves from the active theme's `[fonts]` contribution, if any, and otherwise from the built-in default

#### Scenario: Reset restores typography defaults and clears font choices

- **WHEN** the user resets preferences after changing typography and font families
- **THEN** Source font size returns to 14px, Reading font size returns to 14px, and Paragraph spacing returns to 12px
- **AND** all three font-family preferences clear to the follow-theme state
- **AND** visible document surfaces reflow to those defaults

#### Scenario: Preferences summary includes typography

- **WHEN** the user opens the preferences summary
- **THEN** it reports the current source font size, rendered font size, and paragraph spacing using localized labels and pixel values

## ADDED Requirements

### Requirement: Preferences panel SHALL expose document font family controls

The Preferences panel typography section SHALL include one control per font slot (source, rendered, code) with localized labels consistent with the font-size controls. Each control SHALL present a follow-theme state and an explicit-family state: in the follow-theme state it SHALL indicate that the theme (or default) font applies; activating the control SHALL present a selection list populated from the fonts installed on the machine, each entry rendered in its own family as live preview, plus a follow-theme entry that clears the stored preference. Selecting an entry SHALL apply it immediately to that document plane and persist it. The control SHALL show an advisory warning when the currently stored family (for example hand-edited into `config.toml`) is not among the installed fonts.

#### Scenario: Controls reflect the current slot state
- **WHEN** the Preferences panel is open
- **THEN** each of the three font controls shows either the follow-theme state (with the effective family named) or the user's explicit family for that slot

#### Scenario: The selection list enumerates installed fonts with live previews
- **WHEN** the user opens a slot's font control
- **THEN** the list presents a follow-theme entry followed by every font family installed on the machine, each rendered in its own family
- **AND** the entry matching the slot's current explicit family is marked active

#### Scenario: Selecting a family applies and persists immediately
- **WHEN** the user selects an installed family from a slot's list
- **THEN** that document plane re-renders with the new family on the next frame
- **AND** the choice is written to `config.toml` as the slot's `*_font_family` key
- **AND** the selection list closes

#### Scenario: Follow theme entry clears an explicit choice
- **WHEN** the user selects the follow-theme entry for a slot that had an explicit family
- **THEN** the stored preference is removed so the slot resolves from the active theme and then the default
- **AND** the plane re-renders accordingly and the list closes

#### Scenario: A stored but uninstalled family warns
- **WHEN** a slot's stored family (e.g. hand-edited into `config.toml`) is not installed on the machine
- **THEN** the control shows a localized advisory warning while keeping the value applied and persisted verbatim

#### Scenario: Controls follow language and theme
- **WHEN** the active language or theme changes
- **THEN** font control labels, states, and colors update on the next render
