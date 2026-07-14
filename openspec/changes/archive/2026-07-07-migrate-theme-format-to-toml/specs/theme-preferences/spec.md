## ADDED Requirements

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
