## 1. Preference model and persistence

- [x] 1.1 Add `CodeTheme` (Light/Dark) and the `code_theme`, `code_long_line_wrap`, `code_font_size: Option<_>` fields with defaults (Dark / on / None) to `AppPreferences` in `src/model.rs`, reusing the existing numeric size normalizer family (10–32px clamp) for the explicit code size
- [x] 1.2 Add the `code_theme`, `code_long_line_wrap`, and optional `code_font_size` keys to `PreferencesFile` in `src/storage/preferences.rs` with tolerant deserializers (unknown theme → Dark, missing/non-boolean wrap → on, non-numeric size → absent, out-of-range size → clamp) and wire both `From` impls, omitting the size key when `None`
- [x] 1.3 Extend the preferences round-trip/degradation tests: light theme + wrap off + explicit size round-trip; a config omitting all three keys loads Dark/on/follow-reading; invalid values degrade safely without failing startup

## 2. App state and propagation

- [x] 2.1 Add `code_theme`, `code_long_line_wrap`, and `code_font_size` state fields to `MarkionApp` (`src/app/mod.rs`) with startup mapping and `current_preferences()` output (`src/app/application.rs`)
- [x] 2.2 Add setters in `src/app/appearance.rs`: `set_code_theme`, `toggle_code_wrap`, and `set_code_font_size(Option<_>)` — mutate, emit a localized status, persist, notify — and extend `reset_preferences` to restore Dark / wrap on / line numbers on / size cleared to follow-reading
- [x] 2.3 Resolve the code size in `DocumentTypographyMetrics` (`src/app/mod.rs`): explicit value when set, otherwise `12 × rendered_scale` as today, with `code_line_height` scaling proportionally (19/12); confirm reading-size changes still rescale code when no explicit size is set

## 3. Rendering: code display theme

- [x] 3.1 Introduce the `CodePalette` (chrome + per-`HighlightKind` token colors) with a Dark const copying today's values verbatim and a Light const derived from the PDF-export light token palette plus light chrome neutrals; make `highlight_color`, `code_line_text`, `code_block_text`, `code_block_header`, and `code_copy_button` take the active palette (`src/app/preview.rs`)
- [x] 3.2 Consume the palette in `code_block_view` and `visual_code_editor` (background, default text, gutter, language label, copy affordance) and capture the active code theme into the preview/visual render closures alongside the existing line-numbers capture (`src/app/root_view.rs`, `src/app/preview.rs`)
- [x] 3.3 Confirm the theme switch is presentation-only: no document-version bump, no derived-cache rebuild, and the memoized `highlighted_code` cache stays keyed on (language, code) with colors applied at render time

## 4. Rendering: long-line wrapping

- [x] 4.1 In `code_block_view`, when wrapping is off, host the code rows (both the gutter and joined-text paths) in an `overflow_x_scroll` container with natural-width content following the display-math pattern, keeping exactly one gutter number per logical line (`src/app/preview.rs`)
- [ ] 4.2 Verify text selection and copy still work inside the horizontally scrolled code block (click, drag, copy button) in both the gutter and no-gutter paths, and that `visual_code_editor` continues to soft-wrap regardless of the preference

## 5. Preferences panel UI

- [x] 5.1 Add the new `Msg` variants (Code blocks section header, code highlight theme label, Light, Dark, wrap-long-lines label, code font size label, follow-reading-size action, and toggle status messages) to `src/i18n.rs` with translations for all seven languages and coverage-test list entries, reusing existing `PrefPanelCodeLineNumbers` strings for the relocated toggle
- [x] 5.2 Append the Code blocks section to `preferences_appearance_body` after Typography: a Light/Dark segmented row (option-button pattern), the relocated line-numbers toggle, the wrap toggle, the size stepper showing the effective value with a follow-reading action while an explicit size is stored, and the relocated `FontSlot::Code` font row (`src/app/root_view.rs`)
- [x] 5.3 Remove the code-line-numbers row from the General tab body and the code font-family row from the Typography section, keeping Source/Rendered typography rows untouched

## 6. Verification

- [x] 6.1 Run `cargo test` for the root package (preferences round-trips, i18n coverage, preview tests) and fix any regressions
- [ ] 6.2 Run `cargo test --workspace`, then manually smoke-test across Read, Split Preview, and Visual Edit: all four controls apply immediately and persist; a fresh/older `config.toml` reproduces today's dark wrapped look; invalid hand-edited values degrade safely; reset restores defaults
