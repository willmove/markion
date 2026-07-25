# Design: fidelity-first budgets for preview images

## Context

`PreviewImageCache` owns decoded preview rasters and presents them via `ImageSource::Render`. Two facts shape the design:

- `RenderImage::new` fixes `scale_factor = 1.0` and gpui lays the image out at `pixel_size / scale_factor` **logical** pixels; the field is `pub(crate)`, so Markion cannot mark a bitmap as high-density. A bitmap shown in an N-logical-px slot on a 2× display needs 2·N device pixels of data to stay sharp. The diagram pipeline already works around this: rasterize at 2× and set an explicit presentation width of half the pixel size.
- The completed-byte budget (64 MiB) with never-evict-claimed (anti-flicker, from cef4531) means a pathological all-on-one-screen document can exceed budget with claimed images alone. c89b7e4 answered that by shrinking claimed images; this change answers it by accepting temporary overshoot for claimed images instead, because silently destroying on-screen fidelity is worse than transient memory use — and the pre-c89b7e4 alternative (erroring the incoming image) showed broken previews.

## Goals / Non-Goals

- Goal: an image on screen is never blurrier than the decode cap allows, and never degrades after first paint.
- Goal: tab switches reuse decoded images (no placeholder flash, no re-decode churn).
- Non-goal: exact-to-the-byte budget adherence at every instant when the visible set alone exceeds budget.

## Decisions

### 1. Decode always targets `PREVIEW_IMAGE_MAX_EDGE`

Delete `fair_share_max_edge` from the scheduler. Claim count is a poor proxy for pressure: it counts keys (including pending/error), ignores real image sizes, and punishes quality before any bytes exist. The peak-decode protections (resize on `DynamicImage` before `into_rgba8`, heavy-slot concurrency) already bound transient cost.

### 2. Budget order: evict unclaimed → accept claimed overshoot → last-resort downscale of the incoming raster only

`complete()` keeps: evict unclaimed LRU until it fits. If still over because claimed entries fill the budget, insert the new ready entry anyway (overshoot) up to a hard ceiling (e.g. 2× budget); beyond the ceiling, downscale the incoming raster to the remaining allowance before storing, from its just-decoded bitmap (single resample from the sharpest available data — never resample an already-shrunk entry). No existing entry is ever mutated. Rationale: the incoming image is the one thing not yet on screen, so degrading it is the least-visible compromise, and the overshoot window keeps the common case (a few large images) sharp.

### 3. SVG parity with diagrams

`rasterize_svg_bytes` renders at `2.0 ×` the clamped intrinsic size (reusing the existing edge clamp at `2 × PREVIEW_IMAGE_MAX_EDGE` device pixels). `PreviewImageReady` records the intrinsic (1×) display size; `preview_image_view` sets an explicit `.w(px(display_width))` (and max-w-full) for supersampled entries, exactly like `visual_diagram_editor` does. Raster (non-SVG) images keep implicit sizing — their decoded pixels already exceed typical display slots.

### 4. Release demotes instead of removing

`release()` at claim count zero leaves the entry in the LRU as unclaimed (immediately evictable under capacity or byte pressure). Dormancy and tab close release claims as today; the difference is only that the decoded bytes survive until the budget actually needs them. `cx.drop_image` moves from the release path to the eviction path (it already exists there).

## Risks / Trade-offs

- Overshoot ceiling means worst-case retained bytes can reach ~2× budget while everything is claimed on screen. Bounded and visible in the memory report; preferable to visible blur.
- Keeping unclaimed entries raises steady-state usage toward the budget. That is what the budget is for; diagnostics already report it.
- SVG supersampling quadruples SVG raster bytes; SVGs are typically small (icons/logos), and the edge clamp bounds the worst case.

## Open Questions

- Should decode edge scale with `window.scale_factor()` and observed container width instead of the fixed 2048? Deferred: requires threading window state into the scheduler; 2048 already covers 2× displays at typical widths.
