## 1. Data model and persistence

- [x] 1.1 Add `ThemeFonts { editor, rendered, code: Option<String> }` to `ThemeDefinition` in `src/model.rs` with `Default` = all `None`; extend the fourteen built-in theme entries and the sample theme writer without setting any fonts.
- [x] 1.2 Extend `AppPreferences` and `src/storage/preferences.rs` with optional string keys `editor_font_family`, `rendered_font_family`, `code_font_family`: absent/empty/whitespace-only loads as `None`, values save verbatim, and reset clears all three.
- [x] 1.3 Extend `src/storage/theme_file.rs` to parse and render the optional `[fonts]` table (`editor`, `rendered`, `code`; empty values treated as absent) with round-trip and partial-file tests.
- [x] 1.5 Seed the themes directory on first use with `typewriter.toml` — a sample custom theme whose `[fonts]` table demonstrates per-plane font contributions — replacing the previous Midnight sample, and document it in both READMEs.
- [x] 1.4 Add a pure resolution function (preference over active-theme font over per-slot default) plus unit tests covering all precedence combinations, including empty-string normalization and `".SystemUIFont"` pass-through.

## 2. Rendering application

- [x] 2.1 Add resolved slot state to the app (recomputed on preference edits and on theme application) so every render reads pre-resolved families instead of re-deriving per frame.
- [x] 2.2 Apply the rendered slot on rendered-surface containers (preview list, Visual Edit list) and switch the caret/ascent measurement site in `src/app/preview.rs` from `.SystemUIFont` to the resolved rendered slot.
- [x] 2.3 Apply the source slot to the source editor container so the source surface's inherited text style carries it; verify chrome still inherits `.SystemUIFont` from the root.
- [x] 2.4 Replace the six hardcoded "JetBrains Mono" sites in `src/app/preview.rs` with the resolved code slot, attaching the monospace fallback chain (`FontFallbacks`) to the code slot's resolved font.
- [x] 2.5 On any resolved-family change, run `refresh_typography_measurements(true, true)`; on theme application, re-resolve and invalidate only when an effective family actually changed.

## 3. Invalidation and cache identity

- [x] 3.1 Add the resolved source family to the source measured-height cache key in `src/app/editor_element.rs` so a family-only change can never hit a stale cache entry.
- [x] 3.2 Add a regression test proving a family-only change (same font size) invalidates cached layout heights, preserves scroll position approximately, and does not bump document versions or rebuild derived caches.

## 4. Preferences panel UI and i18n

- [x] 4.1 Add one tri-state font control row per slot to the typography section of the Preferences panel: follow-theme state (naming the effective family), explicit-family text input rendered in that family, and immediate apply/persist on commit.
- [x] 4.2 Add the follow-theme clear action that removes the stored preference and returns the slot to theme/default resolution.
- [x] 4.3 Add advisory unknown-family detection against `TextSystem::all_font_names()` with a localized warning that does not block committing the value.
- [x] 4.4 Add all new user-facing strings (three labels, follow-theme label, warning text, status messages) to `src/i18n.rs` for every supported language and extend the i18n completeness test if it enumerates messages.
- [x] 4.5 Replace the free-form capture input with a scrollable installed-font selection list (follow-theme entry first, per-family live preview, active marker, one list open at a time), fed by `TextSystem::all_font_names()`.

## 5. Tests, docs, and validation

- [x] 5.1 Pure-logic tests for slot resolution, TOML `[fonts]` round-trips, and `config.toml` font-key persistence (no real text shaping with non-system families in unit tests).
- [ ] 5.2 Verify presentation-only behavior manually with multiple tabs: change each slot family and confirm text/dirty state/undo history/selection are untouched while all surfaces reflow.
- [x] 5.3 Update `README.md` / `README.zh-CN.md` preference documentation if it enumerates settings, and mention `[fonts]` in any theme-authoring docs.
- [x] 5.4 Run `cargo test --workspace` and `openspec validate add-configurable-fonts`; fix any failures.
