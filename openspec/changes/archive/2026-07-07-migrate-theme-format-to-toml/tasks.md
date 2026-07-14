# Implementation Plan: Migrate custom theme format to TOML

## Overview

Replace the hand-rolled `.theme` `key=value` format with a serde-derived `.toml` schema, mirroring the `preferences.conf` → `config.toml` migration in `src/storage/preferences.rs:138-164`. Existing `.theme` files migrate on first load and are then left in place, ignored.

## Tasks

- [x] 1. TOML schema (`src/storage/theme_file.rs`)
  - [x] 1.1 Add `ThemeFile` serde struct (`name`, `is_dark`, `[colors]` sub-table of 8 keys) with `#[serde(default)]` so partial files load with the fallback palette.
  - [x] 1.2 Custom color deserializer accepting `"#rrggbb"` or `rrggbb` → `u32`; serializer emitting `"#rrggbb"`.
  - [x] 1.3 `parse_theme_definition` / `render_theme_definition` rewritten over `toml::from_str` / `toml::to_string_pretty`.

- [x] 2. Legacy migration (mirror `load_app_preferences`)
  - [x] 2.1 Rename the old `key=value` reader to `parse_legacy_theme_definition`.
  - [x] 2.2 `load_theme_definition`: if `.toml` exists parse it; else look for sibling `.theme` of same stem, parse legacy, write `.toml`, leave `.theme`, `tracing::info!`.
  - [x] 2.3 `list_theme_definitions`: glob both `.toml` and `.theme`, dedupe by stem (`.toml` wins; orphan `.theme` migrated then listed from the new `.toml`).

- [x] 3. Sample theme + README
  - [x] 3.1 `ensure_sample_custom_theme` (`src/main.rs`) writes `midnight.toml`.
  - [x] 3.2 README custom-theme line updated to `.toml` with auto-migration note.

- [x] 4. Tests + verification
  - [x] 4.1 New `legacy_theme_migrates_to_toml_once` (idempotent migration + legacy-left-in-place + dedupe).
  - [x] 4.2 New `partial_toml_theme_loads_with_fallback_palette`; updated existing round-trip + invalid-values tests to TOML input.
  - [x] 4.3 `cargo test -p markion` green (111 lib + 6 bin tests, 0 failed); `openspec validate migrate-theme-format-to-toml` passes.
