## Why

Custom themes are the one user-facing on-disk format still using a hand-rolled `key=value` text format (`src/storage/theme_file.rs`), even though preferences already moved to TOML (`toml-config-logging-autosave`). Two costs follow: the parsing is bespoke (every field needs its own match arm; errors are line-number strings rather than structured), and the format is inconsistent with the rest of the configuration surface — users who learned `config.toml` meet a different syntax the moment they author a theme.

Mirroring the preferences migration exactly (`preferences.conf` → `config.toml` in `src/storage/preferences.rs:138-164`) keeps the two formats coherent: TOML everywhere, with a one-shot legacy reader that migrates an existing `.theme` file on first load and then leaves it in place, ignored.

## What Changes

- **New TOML schema** (`src/storage/theme_file.rs`): a serde-derived `ThemeFile` struct with `name`, `is_dark`, and a `[colors]` sub-table whose 8 keys (`app_bg`/`panel_bg`/`surface_bg`/`text`/`muted`/`border`/`active_bg`/`active_text`) match `ThemeColors` exactly. Color values are written as `"#rrggbb"` strings and deserialized to `u32` via a custom `serde` helper that accepts both `#rrggbb` and bare `rrggbb` (preserving the legacy reader's leniency). Every field is `#[serde(default)]` so partial files load with the existing fallback palette.
- **Legacy migration**: `load_theme_definition` / `list_theme_definitions` gain the same shape as `load_app_preferences` — if the `.toml` target exists, parse it; else look for a sibling `.theme` with the same stem, parse it with the retired `key=value` reader (renamed `parse_legacy_theme_definition`), write out the `.toml`, leave the `.theme` in place, and `tracing::info!` the migration.
- **Listing**: `list_theme_definitions` globs both `.toml` and `.theme`, dedupes by file stem (`.toml` wins; a `.theme` is only read when no `.toml` of that stem exists, which is exactly the migration case). Built-in shadowing in `available_themes()` (`src/main.rs`) is unchanged.
- **Sample theme**: `ensure_sample_custom_theme` (`src/main.rs`) writes `midnight.toml` instead of `midnight.theme`.
- **README** lines 24-26: the documented custom-theme format and example move from `key=value` to the TOML shape.

## Capabilities

### Modified Capabilities
- `theme-preferences`: custom (user-authored) themes are stored as `.toml` files in the themes directory instead of `.theme` `key=value`; a one-shot migration converts existing `.theme` files on first load and leaves the originals in place.

## Impact

- Edited: `src/storage/theme_file.rs` (rewrite to TOML + legacy migration), `src/main.rs` (`ensure_sample_custom_theme` writes `.toml`), `README.md` (format section).
- New tests: `legacy_theme_migrates_to_toml_once` mirroring `legacy_preferences_migrate_to_toml_once`; the existing sample-theme round-trip test updates its assertion to the `.toml` path.
- No absorbed-crate, config-schema, or UI-surface changes beyond the sample-theme extension.
