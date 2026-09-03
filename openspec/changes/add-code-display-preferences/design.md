## Context

Today every rendered code block is styled by hardcoded constants: `highlight_color` maps `HighlightKind` token classes to one dark palette (`src/app/preview.rs:6315`), and both `code_block_view` (Read / Split Preview, `preview.rs:5638`) and `visual_code_editor` (Visual Edit direct editor, `preview.rs:4579`) fix the chrome at `bg 0x0f172a` / text `0xe2e8f0` with fixed gutter, label, and copy-button colors. Line numbers are gated by the existing `code_line_numbers` preference whose toggle lives on the General tab; code text always soft-wraps; code size is derived (`DocumentTypographyMetrics`: `code_font_size = 12 × rendered_scale`, `code_line_height = 19 × rendered_scale`, `src/app/mod.rs:1583`). Preferences flow `AppPreferences` (`src/model.rs:166`) → `PreferencesFile` (`src/storage/preferences.rs`) → `config.toml`, and the Appearance tab body is `preferences_appearance_body` (`src/app/root_view.rs:4763`). See proposal.md for motivation.

Relevant existing invariants: highlighting is memoized per (language, code) pair and stores token **kinds** only — colors are applied at render time; typography changes are presentation-only (no document-version bump, no derived-cache rebuild, scroll preserved).

## Goals / Non-Goals

**Goals:**

- One place (Appearance → Code blocks) for all code-display controls; every control applies on the next render and persists.
- Zero-regression defaults: absent keys reproduce today's exact appearance (dark, wrapped, line numbers on, size following the reading font).
- Keep presentation-only invariants intact for all four settings.

**Non-Goals:**

- Sticky (non-scrolling) gutter when horizontal scrolling is active; the gutter scrolls with the content.
- No "follow app theme" option for the code theme; Light/Dark is an independent choice.
- No changes to inline code spans, source islands, Markdown Reference examples, or export (PDF/DOCX) code colors.

## Decisions

### D1: A `CodePalette` pair selected at render time, not syntect themes

Add `enum CodeTheme { Dark, Light }` (model + app state, persisted as `code_theme`). Introduce a `CodePalette` struct (`bg`, `text`, `gutter`, `label_accent`, `copy_bg`, `copy_accent`, plus one color per `HighlightKind`) with two const instances: Dark copies today's constants verbatim; Light takes token colors from the existing PDF-export light palette (`pdf_highlight_color`, `src/export.rs:515`) so screen-light and print agree, with GitHub-Light-style neutrals for chrome. `highlight_color(kind)` becomes a palette lookup; `code_line_text` / `code_block_text` / `code_block_view` / `visual_code_editor` / `code_block_header` / `code_copy_button` take the active palette (or `CodeTheme`) as a parameter.

Rationale: the `code-and-math` spec requires colors to stay `HighlightKind`-mapped; because memoized highlighting stores kinds only, a theme switch never invalidates the memoization cache — it is a pure render-time parameter. Alternative rejected: swapping syntect `.tmTheme` files — violates the spec and would re-tokenize.

### D2: Reuse existing preference plumbing for all four settings

Follow the `code_line_numbers` end-to-end pattern exactly: `AppPreferences` field + default → `PreferencesFile` key with tolerant deserializer → `MarkionApp` field → startup mapping + `current_preferences()` → setter in `src/app/appearance.rs` (mutate, localized status, `persist_preferences()`, `cx.notify()`) → restore in `reset_preferences`.

- `code_theme: CodeTheme` — string `light`/`dark`, default Dark, unknown values degrade to Dark.
- `code_long_line_wrap: bool` — default `true`, missing/non-boolean degrade to `true`.
- `code_font_size: Option<f32>` (same numeric type as `editor_font_size`) — absent key means follow-reading; a present value normalizes to 10–32px via the existing clamp-deserializer family. Non-numeric degrades to absent (typography precedent).
- `code_line_numbers` — unchanged key/default; only its UI row moves.

Rationale for `Option` code size: an always-stored value would silently pin existing users (who scaled code via reading size) back to 12px. Absent = derived mirrors the established "follow theme" pattern used for font families.

### D3: Code size resolves inside `DocumentTypographyMetrics`

`DocumentTypographyMetrics::new` gains the optional explicit code size: `code_font_size = explicit.unwrap_or(12 × rendered_scale)`, `code_line_height = round(code_font_size × 19/12)` — preserves today's 12→19 proportion at every size. `set_code_font_size(Option<f32>)` routes through the same `refresh_typography_measurements(false, true)` rendered-change path used by reading-size edits, which already invalidates preview/visual measurements while preserving scroll and document invariants. Clearing (follow-reading action) sets `None` and takes the same path.

### D4: Wrap-off reuses the math-block horizontal-scroll pattern

When wrapping is on (default) nothing changes. When off, `code_block_view` wraps its code content rows in an `overflow_x_scroll()` container whose content gets natural width (`min_w` of measured width, per the display-math pattern around `preview.rs:5979`) and the code text uses no-wrap layout. The line-number gutter stays **inside** the scroll container — one number per logical line is preserved (each logical line is already its own flex row in the gutter path, so wrap state cannot desynchronize numbering). This applies to `code_block_view` only; `visual_code_editor` keeps soft wrap (its single `VisualEditableText` element owns caret/IME geometry; horizontal caret-chasing is out of scope — spec'd in the code-and-math delta).

### D5: Appearance tab section composition

In `preferences_appearance_body`, append a localized "Code blocks" section after Typography: header (`preference_section_header`), a Light/Dark segmented row (reuse the `preference_option_button` pattern from the heading-depth row), the relocated line-numbers and new wrap boolean rows (`preference_boolean_row`), the size stepper (`preference_numeric_row`, showing the **effective** value) plus a small "follow reading size" action visible only while an explicit size is stored, and the relocated `preference_font_row(FontSlot::Code)`. Remove the code-line-numbers row from the General body and the code row from Typography. New `Msg` variants get all 7 language translations plus coverage-test entries; existing `PrefPanelCodeLineNumbers` and its status strings are reused for the relocated toggle.

## Data flow and caching

AppPreferences (load) → `MarkionApp` fields (`application.rs` startup mapping) → render: `root_view` captures `code_theme`/wrap alongside today's `preview_code_line_numbers` capture into the virtualized preview/visual closures → `code_block_view` / `visual_code_editor` read the palette and `typography_metrics()`. Mutations go setter → field + metrics refresh (size only) → persist → `cx.notify()`. Nothing in this flow touches document text, the per-version derived caches (`Arc`), the memoized `highlighted_code` cache, or text handles — all four settings are frame-time render parameters, matching the `document-typography` presentation-only invariants.

## Risks / Trade-offs

- [Selection inside an `overflow_x_scroll` code block misbehaves] → `SelectablePreviewText` must stay correct under horizontal scroll; verify click/drag selection and copy in both gutter and no-gutter paths during implementation, and cover with a test where feasible.
- [Light palette readability] → token colors lifted from the PDF light palette already work on paper-white; still contrast-check every `HighlightKind` on the chosen light chrome during implementation.
- [Fixed 36px gutter overflows past 999 lines] → pre-existing limitation, unchanged by this change.
- [Users expect the Visual Edit editor to honor wrap-off/line numbers] → explicitly spec'd out; its single-element editor cannot host an aligned gutter without per-visual-line layout work (future change candidate).
- [Older builds reading a newer `config.toml`] → unknown keys are ignored/tolerated by the existing reader; an older build re-saving drops the new keys (acceptable downgrade behavior, same as every prior preference addition).

## Migration Plan

Purely additive preference keys with absent-means-current-behavior defaults — no data migration. Rollback = revert the change; `config.toml` keys left behind are ignored by the old code.

## Open Questions

- Final Light palette hex values (chrome neutrals and any token nudges) — decided during implementation against the PDF palette with a contrast pass; does not affect approach, specs, or tasks.
