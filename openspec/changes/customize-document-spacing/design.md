## Context

Paragraph spacing is a normalized `u16` preference limited to 0–32px. Appearance uses the existing numeric minus/plus rows. `DocumentTypographyMetrics` derives different source, prose, list, quote and code line heights; heading and table helpers also derive golden-ratio line heights outside that struct. Typography reflow already invalidates presentation measurements while preserving document-derived caches and virtual-list scroll anchors.

## Goals / Non-Goals

**Goals:** wider paragraph gaps; one understandable relative line-spacing control; old configuration keeps identical metrics; coherent document geometry and persistent/resettable preferences in all eight languages.

**Non-Goals:** export typography, per-document styles, glyph/font-size changes, chrome scaling, or a new preferences-input framework.

## Decisions

1. Expand paragraph spacing to 128px, retain its 12px default and 1px steps. A larger supported bound is preferable to an unbounded gap that could dominate virtualized documents.
2. Store integer `line_spacing` in percent, default 100, bounds 75–300, with 5-point UI steps. The localized label states that 100% means the default. A relative percentage preserves existing plane-specific proportions; one absolute line height would flatten these differences and change old documents. The lower bound keeps default line boxes larger than glyph sizes.
3. Resolve the percentage once into typography metrics. Apply it to source, paragraph, list, quote, source-island and code line heights, plus the shared heading/table text-line helper. Do not scale paragraph gaps or glyph sizes. Default construction remains compatible for existing tests/callers; production construction receives the saved scale.
4. Data flow: `config.toml` → normalized `AppPreferences` → application state → resolved typography → shaping/layout/paint/caret/IME geometry. Preference changes call the existing presentation-only remeasurement for both source and rendered planes, persist settings and notify the UI. Markdown versions, shared derived `Arc`s, highlights and cached text handles are untouched.
5. Reuse the existing numeric controls and localization catalog. Add the preference to initialization, serialization, preferences summary and reset. Missing/wrong-type fields default safely, numeric extremes clamp before conversion.

## Risks / Trade-offs

- Inconsistent painting versus hit testing → route heading/table helper calls through the same scaled metrics and add geometry tests.
- Large gaps or tall rows → bounded values, remeasure virtual-list extents and verify scroll-anchor preservation.
- A percentage mistaken for a font-size multiple → label the default relationship in the UI and describe it in the guide.
- Existing uncommitted README/Korean work → build on the current eight-language catalog and preserve unrelated changes.
