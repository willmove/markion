## Why

Mermaid diagrams reach preview through `Image::from_bytes(ImageFormat::Svg, ..)` plus `img(image).max_w_full()`. The backend's SVG is correct — rasterizing it directly produces accurate flowchart, sequence, class, state, and CJK-labelled output — but GPUI 0.2.2 mishandles this specific path, so the diagram the user sees is wrong in three independent ways.

1. **Colors are inverted between red and blue.** `RenderImage` is documented as BGRA. In `gpui::Image::to_image_data`, the Png/Jpeg/Gif/Webp/Bmp/Tiff branches each swap RGBA to BGRA, but the `ImageFormat::Svg` branch uploads the premultiplied-RGBA `tiny_skia` pixmap unswapped. GPUI's other SVG path (`ImageAssetLoader`) does apply `swap_rgba_pa_to_bgra`, confirming the omission is a defect rather than a convention. Mermaid's light node fill `#ECECFF` is displayed as `#FFECEC` and its `#2F3B4D` edges as `#4D3B2F`, so every diagram renders pink and brown instead of lavender and navy.
2. **Diagrams are rasterized at one device pixel per SVG unit.** The Svg branch uses `SvgSize::ScaleFactor(1.0)` and leaves `RenderImage::scale_factor` at `1.0`, while GPUI's resource SVG path rasterizes at `SMOOTH_SVG_SCALE_FACTOR` (2.0) and records the matching scale factor. Diagram text and strokes are visibly soft, and worse on HiDPI displays.
3. **Diagrams are sized from their raster rather than their intrinsic size.** `img(image).max_w_full()` leaves both dimensions auto, so GPUI resolves them from `render_size`. That is harmless at 1x today, but it makes explicit sizing a precondition for fixing (2): once the raster is supersampled, an auto-sized element would draw every diagram at double size. The same auto-sizing also leaves `max_w_full()` constraining width only, so a wide diagram keeps its intrinsic height and `ObjectFit::Contain` letterboxes it inside a too-tall box.

GPUI does not export `SvgRenderer`, `SvgSize`, or `SMOOTH_SVG_SCALE_FACTOR` (`gpui.rs` imports the module with `use svg_renderer::*;`, not `pub use`), so Markion cannot reuse or configure GPUI's SVG decode. Correct presentation requires Markion to rasterize sanitized diagram SVG itself and hand GPUI a ready `RenderImage`.

## What Changes

- Rasterize sanitized diagram SVG inside the existing background render task, producing a BGRA `RenderImage` supplied to `img()` through `ImageSource::Render` instead of `Image::from_bytes(ImageFormat::Svg, ..)`.
- Rasterize at a fixed 2x supersample and present the diagram at its sanitized intrinsic size. `RenderImage::scale_factor` is `pub(crate)`, so GPUI would otherwise size every diagram from the raw supersampled pixel count and draw it at double size.
- Load system fonts into the rasterizer's `usvg` options. `usvg::Options::default()` carries an empty font database, which silently drops every `<text>` node and would erase all diagram labels.
- Keep a diagram wider than its preview column fully visible and undistorted. Such a diagram still reserves its intrinsic height, because GPUI makes both image dimensions definite before layout and taffy's aspect-ratio rule can only grow a height, never shrink it; removing that residual padding needs container-width measurement and is left out of scope (see `design.md`).
- Cache the rasterized image and its presentation size together, keeping the existing `(backend_id, source, theme)` key, pending/ready/error states, deduplication, and bounded eviction.
- Non-goals: replacing `mermaid-rs-renderer`, widening Mermaid syntax coverage, changing the sanitization boundary, adding diagram zoom/pan/export affordances, or altering HTML export (which inlines SVG and never touches this raster path).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `diagram-rendering`: Require rendered diagrams to reach preview with the backend's authored colors, at display resolution, and with preserved aspect ratio.

## Impact

- Affected code: `src/app/diagram.rs` (rasterization plus cache entry payload) and the diagram branch of `src/app/preview.rs`.
- Adds direct root-crate dependencies on `resvg`, `usvg`, and `image`, all already resolved in `Cargo.lock` as GPUI's own transitive dependencies, so versions unify and `image::Frame` stays compatible with `RenderImage::new`.
- Preserves the `crates/*` GUI-free invariant: rasterization produces `gpui::RenderImage` and therefore stays in the root crate; `markion-diagram` continues to return sanitized SVG only.
- Preserves the per-document-version derived Markdown caches. Diagram rasterization remains keyed by immutable content, runs on the background executor, and does not reparse or bump the document version.
- Depends on `add-mermaid-diagrams` being archived first, since it owns the `diagram-rendering` capability this change modifies.
