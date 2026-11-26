## Why

On Linux desktops that do not have `IBM Plex Sans` installed (the default target of GPUI's `.SystemUIFont`), every request for bold, semibold, italic, oblique, or bold-italic text fails primary lookup and falls back to `TextSystem::fallback_font_stack`, which discards the requested `FontWeight`, `FontStyle`, `FontFeatures`, and `FontFallbacks`. As a result, rendered Markdown bold/italic text, headings, definition terms, UI semibold labels, and code blocks (`JetBrains Mono` falling back past `DejaVu Sans Mono` to proportional `Noto Sans`) all silently degrade to upright regular proportional text. Additionally, GPUI's Linux `CosmicTextSystem` passes `CacheKeyFlags::empty()` to `swash`, so italic text rendered through fonts lacking a dedicated italic face (such as CJK fonts or `DejaVu Sans`) renders upright, and Visual Edit uses booleans rather than depth counters for nested strong/emphasis/strikethrough delimiters.

## What Changes

- Preserve `FontWeight`, `FontStyle`, and `FontFeatures` across font fallback resolution in GPUI `TextSystem::resolve_font`, and honor explicit `Font.fallbacks` before falling back to `fallback_font_stack` so code blocks resolve to an installed monospace fallback (such as `DejaVu Sans Mono`, `Liberation Mono`, or `Noto Sans Mono`) instead of proportional sans-serif.
- Update GPUI's Linux `CosmicTextSystem` to retain the requested style and weight on loaded fonts, pass requested weight into `cosmic_text::Attrs` for fallback weight matching (e.g. CJK bold), allocate distinct `FontId`s for synthetic-italic variants to avoid glyph-atlas aliasing, and pass `CacheKeyFlags::FAKE_ITALIC` to `swash` whenever italic/oblique is requested on a normal-style primary or fallback face.
- Extend `code_slot_font`'s monospace fallback chain with `"Liberation Mono"` and `"Noto Sans Mono"` after `"DejaVu Sans Mono"` for broader Linux monospace coverage.
- Replace boolean flags with depth counters (`usize`) for `Tag::Strong`, `Tag::Emphasis`, and `Tag::Strikethrough` in Visual Edit inline run derivation (`src/visual.rs`) so nested same-kind delimiters preserve outer styling after an inner construct closes.
- **Non-goals:** synthetic bold (faux emboldening) in `swash`, CJK CommonMark flanking-delimiter extensions, or Markdown preview/table/HTML/extended-inline parser changes (handled in a separate change).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `document-typography`: require font-family fallback resolution to preserve requested weight/style and honor explicit monospace fallback chains on Linux, and require Linux italic rendering to synthesize oblique glyphs when the matched primary or fallback face has no italic variant.
- `markdown-editing`: require Visual Edit to preserve outer bold, italic, and strikethrough styles across nested same-kind Markdown constructs.

## Impact

- `vendor/zed/crates/gpui/src/text_system.rs`: preserve font style/weight/features in `TextSystem::resolve_font` and resolve `font.fallbacks` before `fallback_font_stack`.
- `vendor/zed/crates/gpui/src/platform/linux/text_system.rs`: track requested weight/style and `fake_italic` per `LoadedFont`, propagate weight/italic intent across `layout_line` fallback runs, and pass `CacheKeyFlags::FAKE_ITALIC` in `raster_bounds` and `rasterize_glyph`.
- `src/app/mod.rs`: append Linux monospace fallbacks to `code_slot_font`.
- `src/visual.rs`: use depth counters for strong, emphasis, and strikethrough inline runs.
- Preserves all Markion architecture invariants: no changes to per-version derived Markdown caching, memoized syntax highlighting, cached text handles, or GPUI-free workspace member crates.
