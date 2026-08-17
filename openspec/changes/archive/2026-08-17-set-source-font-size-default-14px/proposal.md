# Proposal: set-source-font-size-default-14px

## Why

The source editor currently defaults to a 15px font while the rendered reading view defaults to 14px, so a fresh install shows source text one pixel larger than the reading view for identical content. Defaulting the source view to 14px aligns both surfaces and matches the size users asked for as the comfortable baseline.

## What Changes

- Change `DEFAULT_EDITOR_FONT_SIZE` from `15` to `14` (logical pixels) in `src/model.rs`.
- The new default applies wherever the constant is used today: fresh installs with no `config.toml`, configs whose `editor_font_size` field is missing or non-numeric, and preferences reset.
- Source-editor line height is derived as `font_size * (EDITOR_LINE_HEIGHT / DEFAULT_EDITOR_FONT_SIZE)`; to keep the historical 1.6× ratio (and the 22.4px default line height) the nominal `EDITOR_LINE_HEIGHT` constant changes from 24 to 22.4 (= 14 × 1.6) alongside the default.
- Update the one test that hardcodes the 15px default value.
- Existing users who ever changed the source font size have an explicit `editor_font_size` line in `config.toml` and are unaffected; users still on the 15px default (field never persisted) silently move to 14px.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `document-typography`: The "Configurable source-editor font size" requirement changes the source-editor font-size default from 15px to 14px (range stays 10–32px).
- `theme-preferences`: The "Document typography preferences SHALL persist safely" requirement changes the source-font-size default from 15px to 14px; its default-value and reset scenarios are updated accordingly.

## Impact

- `src/model.rs` — `DEFAULT_EDITOR_FONT_SIZE` constant (bounds `MIN_EDITOR_FONT_SIZE`/`MAX_EDITOR_FONT_SIZE` stay 10–32; 14 is well within range).
- `src/app/tests.rs` — one assertion on the default source font size.
- `openspec/specs/theme-preferences/spec.md` — synced at archive time via the delta in this change.
- No i18n changes (no user-facing strings change; the Preferences panel shows the live value).
- No persistence-format change: field name `editor_font_size`, normalization, and clamping are untouched.
- Does not touch the derived-state caching invariants; this is a single constant plus derived scaling that already flows through the existing pipeline.

## Non-goals

- Not changing the reading font size default (already 14px), paragraph spacing default, or the 10–32px clamp bounds.
- Not migrating existing `config.toml` files that already persist an explicit `editor_font_size`.
