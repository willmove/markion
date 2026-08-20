## Why

Typography customization today covers only sizes and spacing — font families are hardcoded: the platform system UI font for prose and the Markdown source editor, and a bare "JetBrains Mono" for code surfaces with no fallback chain (on machines without it, code silently degrades to a proportional font). Users writing long-form prose, preferring a monospace source view, or wanting a theme to carry a typographic identity have no way to choose fonts.

## What Changes

- Introduce three per-plane font-family **slots**: **source** (Markdown source editor), **rendered** (Visual Edit / preview / read body text, including inline code spans), and **code** (fenced code blocks, Visual Edit source islands, reference-definition source views).
- Resolve each slot as **explicit preference → active theme → built-in default** ("follow theme" is the default preference state).
- Persist optional keys `editor_font_family`, `rendered_font_family`, `code_font_family` in `config.toml` (absent = follow theme/default).
- Extend custom-theme TOML with an optional `[fonts]` table (`editor`, `rendered`, `code`), so a theme can ship a typographic identity that applies whenever the user has not made an explicit choice.
- Add three font controls to the Preferences panel typography section with a follow-theme state, unknown-family advisory warning, and live preview.
- Give the code slot a monospace fallback chain so code never degrades to a proportional font.
- Application chrome (menus, sidebar, tab bar, panels) is **unchanged** and keeps the platform system UI font.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `document-typography`: New requirement covering per-plane font-family slots, their resolution precedence, defaults, the code fallback chain, presentation-only invalidation semantics, and the chrome exclusion.
- `theme-preferences`: The custom-theme TOML format gains an optional `[fonts]` table; the typography persistence contract gains three optional string keys with follow-theme/reset semantics; the Preferences panel gains document font family controls.

## Impact

- `src/model.rs`: `ThemeFonts` on `ThemeDefinition`; three optional `Option<String>` fields on `AppPreferences`.
- `src/storage/preferences.rs`: load/save of the three new keys (absent/empty = `None`), included in reset.
- `src/storage/theme_file.rs`: `[fonts]` parse/render plus round-trip tests.
- `src/app/root_view.rs`: per-plane font application replaces the single inherited root family for document surfaces (root keeps the system font for chrome).
- `src/app/preview.rs`: the six hardcoded "JetBrains Mono" sites move to the code slot with a fallback chain; the caret/ascent measurement site resolves the rendered slot.
- `src/app/editor_element.rs`: the source measured-height cache key gains the resolved source family.
- `src/app/appearance.rs`: font changes reuse `refresh_typography_measurements(true, true)`; theme application re-resolves slots.
- `src/i18n.rs`: labels/warnings for the three controls in every supported language.
- No new dependencies; the vendored gpui crate already provides `TextSystem::all_font_names()` and `FontFallbacks`.
- Invariant touched: presentation-only typography invalidation (no document-version bump, no derived-cache rebuilds) must hold for family changes exactly as it does for size changes today.

## Non-goals

UI-chrome font configuration, font weight/feature controls, bundling/embedding font files, per-theme line-height tuning, injecting user fonts into HTML export, and per-script fallback configuration.
