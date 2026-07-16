## Context

`markion-diagram` returns sanitized passive SVG plus an optional intrinsic size. Today `src/app/diagram.rs` wraps those bytes in `Image::from_bytes(ImageFormat::Svg, ..)`, caches `Arc<Image>`, and `src/app/preview.rs` renders `img(image).max_w_full()`. GPUI decodes that lazily on the frame path via `ImageSource::Image` -> `ImageDecoder::load` -> `Image::to_image_data`.

That decode is where fidelity is lost. `to_image_data`'s `ImageFormat::Svg` branch neither converts `tiny_skia`'s premultiplied RGBA to the BGRA that `RenderImage` documents, nor supersamples the way GPUI's own resource-SVG loader does. Both behaviors are internal to GPUI 0.2.2 and unreachable through its public API: `SvgRenderer`, `SvgSize`, and `SMOOTH_SVG_SCALE_FACTOR` live behind a private `use svg_renderer::*;` in `gpui.rs`, and `RenderImage::scale_factor` is `pub(crate)`.

`ImageSource::Render(Arc<RenderImage>)` is public, is returned as-is by `use_data` with no decode step, and `RenderImage::new` is public. That is the seam this change uses.

The revised flow:

```text
markion-diagram: sanitized SVG + intrinsic size
        |
        v
background executor (existing spawn, unchanged key)
        |
        +-- usvg parse with system fontdb  (empty fontdb drops all <text>)
        +-- resvg render at 2x supersample
        +-- premultiplied RGBA -> BGRA
        |
        v
cache: Ready(Arc<RenderImage>, presentation size)
        |
        v
preview: img(ImageSource::Render).max_w_full() with aspect-correct sizing
```

## Goals / Non-Goals

**Goals:**

- Present diagrams with the colors, resolution, and proportions the backend authored.
- Keep rasterization on the background executor, keyed by immutable content, off the GPUI frame path.
- Keep the sanitization boundary authoritative: only sanitized bytes are ever rasterized.
- Keep `markion-diagram` GUI-free.

**Non-Goals:**

- Fixing GPUI upstream, vendoring it, or patching `to_image_data`.
- Changing Mermaid syntax coverage, the backend crate, or HTML export.
- Diagram zoom, pan, click, or raster export.
- Per-window DPI-adaptive supersampling; a fixed 2x matches what GPUI itself chose for SVG resources.

## Decisions

### Rasterize in the application crate and hand GPUI a finished `RenderImage`

`RenderImage` is a GPUI type, and the workspace invariant forbids `crates/*` from depending on `gpui`, so rasterization belongs in the root crate next to the existing cache in `src/app/diagram.rs`. `markion-diagram` keeps returning sanitized SVG and stays testable headless.

The root crate gains `resvg`, `usvg`, and `image` as direct dependencies. All three are already in `Cargo.lock` as GPUI's transitive dependencies (`resvg` 0.45.1, `usvg` 0.45.1, `image` 0.25.10), so Cargo unifies them and the `image::Frame` we construct is the same type `RenderImage::new` expects. A future GPUI upgrade that moves those versions must move ours in lockstep; a mismatch surfaces as a compile error at `RenderImage::new`, not as silent corruption.

Alternative considered: reuse GPUI's `SvgRenderer`. Not expressible — the type cannot be named outside `gpui`.

Alternative considered: pre-swap red and blue in the SVG source so GPUI's missing swap cancels out. This would fix color while leaving resolution and layout broken, invert exported HTML if the two paths ever shared bytes, and silently produce doubly-wrong output the moment GPUI fixes the bug.

### Load system fonts explicitly into `usvg::Options`

`usvg::Options::default()` holds an empty `fontdb`. Because usvg converts text to paths at parse time, an empty database drops every `<text>` node without an error — a diagram would render as unlabelled boxes and arrows. This was confirmed by probing the sanitized SVG with default options: zero text nodes survived.

The rasterizer therefore builds its options with a `fontdb` populated by `load_system_fonts()`, resolved once in a `LazyLock` and shared by every render. `gpui::SvgRenderer::new` reaches the same state through a custom `select_font` closure that swaps in a lazily loaded database; setting `Options::fontdb` directly is equivalent and simpler, because usvg's default selector queries exactly that field. Font loading is expensive and happens off the frame path on first use.

**Known limitation:** usvg resolves a generic family (`sans-serif`) through fontdb's configured generic names, which default to faces such as `FreeSans` that a system need not have installed — when that lookup fails the text is dropped, exactly as an empty database would. Mermaid's output always names concrete families before its generic fallback (`trebuchet ms,verdana,arial,DejaVu Sans,Liberation Sans,sans-serif,…`), so a host with any one of them renders labels correctly, and GPUI's own SVG rendering has identical behavior. Pinning fontdb's generic families to a face known to be present would harden this, but it is a font-policy decision beyond this change.

### Supersample at 2x and present at intrinsic size

`RenderImage::scale_factor` is `pub(crate)`, so a consumer cannot tell GPUI "these pixels are 2x". Instead the cache stores the presentation size alongside the image, and preview sizes the element explicitly. GPUI's `render_size` reports the raw 2x pixel count, so leaving the element auto-sized would draw every diagram at double size; explicit sizing is what makes the supersample a quality improvement rather than a scaling bug.

### Size by intrinsic width and let GPUI derive height

Supersampling makes explicit sizing mandatory. GPUI resolves an auto image dimension from `render_size`, which — with `scale_factor` stuck at 1.0 — reports the raw 2x pixel count. Leaving the element auto-sized would therefore draw every diagram at double size. Preview sets a definite width equal to the sanitized intrinsic width and leaves height auto, so GPUI computes `height = raster_h * width / raster_w`; the 2x factor cancels and the element lands at intrinsic height.

A diagram wider than its column is clamped by `max_w_full()` and drawn to fit by the default `ObjectFit::Contain`, so it stays fully visible and undistorted.

**Known residual:** such a diagram still reserves its full intrinsic height, leaving vertical padding above and below. GPUI resolves both image dimensions to definite lengths before handing the style to taffy, which makes taffy's `maybe_apply_aspect_ratio` a no-op (it only fills a dimension when exactly one is `None`), and taffy's other aspect-ratio rule in `compute_leaf_layout` is `height = f32_max(clamped_height, clamped_width / ratio)` — it can only grow the height, never shrink it to match a clamped width.

Two ways to remove the padding were considered and rejected. Setting `height` to zero so that `f32_max` rule derives it from the clamped width does produce exactly the right box, but it is obscure and its failure mode is catastrophic rather than cosmetic: any change to that taffy rule or to GPUI's unconditional `aspect_ratio` assignment collapses every diagram to nothing. Measuring the container to compute the fit directly is correct but needs a measured-layout or canvas wrapper inside the virtualized preview list, and this repo has no GPUI layout-test harness to verify one against. Trading a cosmetic gap for a possible invisible-diagram regression is a bad bargain in an editor, so the padding stays until container measurement is done deliberately.

### Keep the cache contract

The key stays `(backend_id, source, theme)`; states stay pending/ready/error; deduplication, FIFO eviction of completed entries, and the rule that in-flight work is never evicted are unchanged. Only the `Ready` payload grows from `Arc<Image>` to the rasterized `Arc<RenderImage>` plus its presentation size. Stale completions remain key-scoped and still cannot touch document state.

## Risks / Trade-offs

- [`resvg`/`usvg`/`image` drift out of sync with GPUI's copies] -> Versions are pinned through the shared lockfile and unify today; a GPUI bump is reviewed together with these dependencies, and a true mismatch fails to compile.
- [A future GPUI release fixes the Svg branch] -> This change stops using that branch entirely, so an upstream fix is inert rather than double-applied.
- [2x supersampling doubles diagram memory] -> Bounded by the existing 128-entry cache and the 4 MiB sanitized-output limit; rasters are cached per immutable key and shared across tabs.
- [System font loading costs time on first diagram] -> Paid once, lazily, on a background thread, never on the frame path.
- [Missing CJK or fallback fonts on a user's system] -> Behavior matches GPUI's own SVG font resolution, including its fallback selector, so diagrams degrade no worse than the rest of the application's SVG rendering.

## Migration Plan

1. Add the root dependencies and a rasterization helper with unit coverage for channel order, supersampling, and text survival.
2. Move the cache payload to the rasterized image plus presentation size.
3. Update the preview diagram branch to size by width with derived height.
4. Run targeted diagram tests, the root suite, the workspace suite, fmt, and clippy.

Rollback restores `Image::from_bytes` and the previous cache payload; no document, spec, or on-disk format depends on this change.

## Open Questions

None blocking. If GPUI later exports its SVG renderer or a settable image scale factor, this rasterizer can be deleted in favor of it without changing the cache or preview contracts.
