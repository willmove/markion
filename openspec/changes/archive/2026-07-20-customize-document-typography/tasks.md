## 1. Preference Model and Persistence

- [x] 1.1 Add documented defaults, bounds, and normalization helpers for source font size, rendered font size, and paragraph spacing, then extend `AppPreferences` with values that preserve the current 15px/14px/12px appearance.
- [x] 1.2 Extend TOML serialization/deserialization with `editor_font_size`, `rendered_font_size`, and `paragraph_spacing`, including tolerant missing/non-numeric handling, numeric clamping, legacy-file defaults, and round-trip coverage.
- [x] 1.3 Plumb normalized typography values through application startup, `MarkionApp` state, current-preference saving, preference reset, and the localized preferences summary without changing per-tab document state.

## 2. Typography Metrics and Rendering

- [x] 2.1 Introduce a root-crate `DocumentTypographyMetrics` value derived from normalized preferences, with source line height and rendered body/heading/list/quote/table/code/math metrics that reproduce current values at the defaults.
- [x] 2.2 Apply source typography metrics to Edit and Split Preview shaping, wrapping, painting, line measurement, caret/selection hit testing, scroll extents, focus mode, and typewriter positioning.
- [x] 2.3 Apply rendered typography metrics across Visual Edit, Split Preview, and Read mode blocks, including source-backed islands, headings, paragraphs, lists, block quotes, tables, code, and inline/display math; use the configured paragraph gap only after rendered paragraph blocks.
- [x] 2.4 Refresh only editor line measurements and virtualized visual/preview row geometry when typography changes, preserving approximate scroll fractions, document versions, derived `Arc` caches, memoized highlights, cached shared text, dirty state, undo/redo history, and selections.

## 3. Preferences Controls and Localization

- [x] 3.1 Add a reusable theme-aware numeric stepper row with localized Source font size, Reading font size, Paragraph spacing, pixel-unit, decrement, and increment labels; disable actions at configured bounds.
- [x] 3.2 Wire the three controls to immediate normalized app setters that persist only real changes, request reflow/repaint, and remain correct after language/theme switches and preference reset.

## 4. Verification

- [x] 4.1 Add focused tests for default/missing/invalid/out-of-range preference values, TOML round trips, normalization, metrics at default and boundary sizes, control-bound behavior, and presentation-only cache/document invariants.
- [x] 4.2 Add or extend GPUI tests covering non-default font wrapping, caret/selection geometry, paragraph gaps, virtual-list measurement refresh, and scroll-position preservation across the affected view modes.
- [x] 4.3 Run `cargo fmt --check` and `cargo test --workspace`, resolving regressions while preserving the per-version Markdown caches, syntax-highlight memoization, cached text handle, and GPUI-free workspace-member boundary.
- [x] 4.4 Build and launch Markion; verify default/minimum/maximum values, immediate controls, restart/reset, long-document scrolling, caret/selection/IME alignment, all four view modes, representative light/dark themes, narrow windows, and representative Windows scale factors.
- [x] 4.5 Run `openspec validate customize-document-typography` and resolve all reported issues before handoff.
