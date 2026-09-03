## Why

Fenced code blocks are rendered with a hardcoded dark chrome and a single dark token palette (`highlight_color`, `code_block_view`, `visual_code_editor`), and the code font size is only indirectly adjustable via the reading font size. Users reading documents on a light app theme get an always-dark code block, and have no control over line-number visibility (the existing toggle is buried on the General tab), long-line wrapping, or code text size. This change groups all code-display controls under the Preferences → Appearance tab so code rendering becomes a first-class appearance setting.

## What Changes

- Add a **Code blocks** section to the Preferences panel **Appearance** tab, containing:
  - **Code highlight theme** — segmented Light / Dark choice (default **Dark**, preserving today's look). Applies to the token palette and the block chrome (background, text, gutter, language label, copy button) on every rendered code-block surface: Read mode, Split Preview's rendered pane, and the Visual Edit direct code editor.
  - **Show line numbers** — the existing `code_line_numbers` toggle, relocated from the General tab's display settings into this section. Behavior scope is unchanged (Read mode and Split Preview rendered pane).
  - **Wrap long lines** — new toggle (default on). Off disables soft wrapping on reading surfaces and exposes the full block content through bounded horizontal scrolling instead; the Visual Edit direct code editor always soft-wraps (it is an editing surface; horizontal caret geometry stays untouched).
  - **Code font size** — new numeric control. Absent preference keeps today's behavior (code size derived from the reading font size, 12px at the 14px default); an explicit value clamps to 10–32px and code line height scales proportionally. A follow-reading-size action clears the explicit value.
- Move the existing **Code** font-family row from the Typography section into the new Code blocks section so all code-display controls live together (Source/Rendered family rows stay in Typography).
- Persist the new preferences in `config.toml` (`code_theme`, `code_long_line_wrap`, optional `code_font_size`) alongside the existing `code_line_numbers`, with tolerant defaults for missing/invalid values and reset behavior.

**Non-goals:** no additional syntax color schemes beyond Light/Dark (palettes stay mapped from `HighlightKind` token classes, never from syntect themes); no "auto / follow app theme" code theme option; no change to inline code spans (they follow the rendered slot per `document-typography`); no change to PDF/DOCX export code colors; no line-number gutter or wrap-off horizontal scrolling inside the Visual Edit direct code editor; no per-document or per-language overrides.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `theme-preferences`: Appearance tab gains the Code blocks section (highlight theme choice, relocated line-numbers toggle, wrap toggle, code font size stepper, relocated code font-family control); persistence contract gains the `code_theme`, `code_long_line_wrap`, and optional `code_font_size` keys with defaults, tolerant parsing, and reset behavior; the typography font-family requirement loses the code slot to the new section.
- `code-and-math`: code-block rendering derives token and chrome colors from the selected Light/Dark code display theme instead of a hardcoded dark palette; a new wrapping requirement makes long-line soft-wrap a preference with horizontal-scroll fallback.
- `document-typography`: code text and line metrics derive from an independent optional code font-size preference (falling back to today's reading-size derivation) instead of always from the rendered body size.

## Impact

- `src/model.rs` — `AppPreferences` fields + defaults (`code_theme`, `code_long_line_wrap`, optional `code_font_size`), size constants/normalizer.
- `src/storage/preferences.rs` — `PreferencesFile` keys, tolerant deserializers, both `From` impls, round-trip tests.
- `src/app/mod.rs` — `MarkionApp` state, `DocumentTypographyMetrics` code size/line-height resolution.
- `src/app/application.rs` — startup mapping, `current_preferences()`.
- `src/app/appearance.rs` — setters/toggles, `reset_preferences` restore, typography refresh wiring.
- `src/app/root_view.rs` — Appearance tab section + rows; General tab row removal; preview list closure captures.
- `src/app/preview.rs` — `highlight_color` palette split, `code_block_view` / `visual_code_editor` chrome + gutter + wrap-off scrolling (math-block `overflow_x_scroll` pattern).
- `src/i18n.rs` — new `Msg` variants with all 7 language translations + coverage-test list entries.
- Invariants preserved: code-display changes are presentation-only (no document version bump, no derived-cache rebuild, memoized highlighting untouched — color mapping stays render-side, scroll position preserved on remeasure); user-facing strings go through `src/i18n.rs`.
