## Context

Markion resolves its default `editor` and `rendered` font slots to `".SystemUIFont"` and its `code` slot to `"JetBrains Mono"` with an explicit `FontFallbacks` list. On Linux, vendored GPUI (`vendor/zed/crates/gpui`) uses `CosmicTextSystem` backed by `cosmic-text 0.14.2` (`fontdb` + `rustybuzz` + `swash`).

Two defects in vendored GPUI interact on Linux systems without `IBM Plex Sans` or `JetBrains Mono`:
1. `TextSystem::resolve_font` checks only `self.font_id(font)` and then `self.font_id(fallback)` for each static `Font` in `fallback_font_stack` (all `weight: NORMAL, style: Normal`). It ignores `font.fallbacks` completely and drops `font.weight`, `font.style`, and `font.features` on fallback.
2. `CosmicTextSystem` binds `FontId` 1:1 to a `fontdb::ID` without storing requested weight/style or a `fake_italic` flag. `layout_line` passes the matched face's `style`/`weight` to `cosmic_text::Attrs`, and `raster_bounds`/`rasterize_glyph` always pass `CacheKeyFlags::empty()`.

Separately, `src/visual.rs` tracks `Tag::Strong`, `Tag::Emphasis`, and `Tag::Strikethrough` with boolean fields on `markdown_style`, whereas `src/lib.rs` (`InlineStateDraft`) uses `usize` depth counters.

## Architecture & Data Flow

### 1. Style-preserving font fallback (`TextSystem::resolve_font`)
- Step 1: Try `self.font_id(font)`.
- Step 2: If `font.fallbacks` is present, iterate each family name, cloning `font` with `family = fallback_family` (preserving `weight`, `style`, `features`, `fallbacks`), and return the first `Ok(FontId)`.
- Step 3: Iterate `self.fallback_font_stack`, first trying `Font { family: fallback.family, ..font.clone() }` so requested weight/style/features survive fallback; if that fails for a given fallback family, try `self.font_id(fallback)` as a final safety net.
- Caching: `TextSystem::font_ids_by_font` already caches every queried `Font` key under a `RwLock`, so subsequent frames pay zero `fontdb` queries.

### 2. Linux `CosmicTextSystem` requested style/weight & synthetic italic
- Extend `LoadedFont` with:
  - `requested_weight: cosmic_text::Weight`
  - `requested_style: cosmic_text::Style`
  - `fake_italic: bool`
- Maintain a lookup table `variant_font_ids: HashMap<(cosmic_text::fontdb::ID, CosmicFontFeatures, cosmic_text::Weight, cosmic_text::Style, bool), FontId>` (or a compact struct key) so distinct `(face_id, features, requested_weight, requested_style, fake_italic)` combinations reuse a single `FontId` without unbounded `loaded_fonts` growth.
- **Glyph-atlas isolation**: Because GPUI caches glyph raster bounds and GPU atlas tiles by `RenderGlyphParams { font_id, glyph_id, ... }`, a face rendered upright (`fake_italic = false`) and the same `fontdb::ID` rendered with `FAKE_ITALIC` (`fake_italic = true`) MUST have distinct `FontId`s. The variant key guarantees this invariant for both primary fonts and fallback fonts.
- In `CosmicTextSystem::font_id`:
  - Match the best face in the requested family via `font_kit::matching::find_best_match`.
  - Compute `wants_italic = matches!(req_style, Style::Italic | Style::Oblique)` and `face_is_italic = matches!(face.style, Style::Italic | Style::Oblique)`.
  - `fake_italic = wants_italic && !face_is_italic`.
  - Effective `requested_weight`: keep `req_weight` when `req_weight.0 >= 600 && face.weight.0 < 600` (so fallback fonts can still select Bold when the primary lacks Bold); otherwise use `face.weight` so `cosmic_text` weight sorting agrees with `font_kit`'s primary choice.
  - Effective `requested_style`: store `req_style` on `LoadedFont` (so `layout_line` knows whether fallback glyphs want italic).
- In `CosmicTextSystemState::layout_line`:
  - Note that `cosmic-text 0.14.2`'s `Attrs::matches(face)` strictly filters by `face.style == self.style && face.stretch == self.stretch`. Therefore, pass `face.style` to `Attrs::style(...)` so `cosmic-text` never rejects the matched primary family when that family lacks an italic face; pass `loaded_font.requested_weight` to `Attrs::weight(...)` so weight sorting selects bold fallback faces (e.g., `Noto Sans CJK SC Bold`).
  - For each shaped glyph in `layout.glyphs`:
    - Determine `wants_italic` from `run_loaded.requested_style`.
    - If `glyph.font_id != run_loaded.font.id()`, look up/create the fallback `FontId` via `font_id_for_cosmic_id(glyph.font_id, wants_italic)`: if the fallback face's own `style` is `Normal` and `wants_italic` is true, mark the fallback `LoadedFont` with `fake_italic = true`.
- In `raster_bounds` and `rasterize_glyph`:
  - Derive `flags = if loaded.fake_italic { CacheKeyFlags::FAKE_ITALIC } else { CacheKeyFlags::empty() }` and pass `flags` to `CacheKey::new(...)`.

### 3. Visual Edit nested delimiter counters (`src/visual.rs`)
- Replace `markdown_style.bold`, `markdown_style.italic`, and `markdown_style.strikethrough` mutations with `bold_depth: usize`, `italic_depth: usize`, and `strike_depth: usize` incremented on `Event::Start` and `saturating_sub(1)` on `Event::End`, mirroring `InlineStateDraft` in `src/parse.rs`/`src/lib.rs`.
- Per-version derived block caching (`visual_blocks` cached per document version) is completely unchanged.

## Verification Plan
- Unit tests in `src/visual.rs` verifying nested `**outer **inner** tail**`, `*outer *inner* tail*`, and `~~outer ~~inner~~ tail~~` preserve outer formatting in Visual Edit projections.
- Linux-gated integration tests exercising the real GPUI `TextSystem` + `CosmicTextSystem`:
  - `.SystemUIFont` resolves distinct `FontId`s for Regular, Bold, Italic, and BoldItalic on Linux even when `IBM Plex Sans` is absent.
  - `code_slot_font("JetBrains Mono")` resolves to an installed monospace fallback (`advance('i') == advance('W')`).
  - Requesting Italic on a normal-only font (e.g. `DejaVu Sans` or `Noto Sans CJK SC`) produces a distinct `FontId` from Regular and produces non-identical raster bounds / rasterized bitmaps (verifying `FAKE_ITALIC` is active and does not alias the upright glyph in the cache).
  - Full `cargo test --workspace` passes.
