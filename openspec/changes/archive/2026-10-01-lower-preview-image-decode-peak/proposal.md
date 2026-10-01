## Why

Process-footprint diagnostics (`report-process-memory-footprint`) confirmed that closing image-heavy tabs can leave resident memory high even when Markion's `PreviewImageCache` correctly releases retained rasters. A decode-spike probe showed `peak ≫ after-drop current ≫ before`: the preview decode path expands photographs to full-resolution intermediates (`DynamicImage` → full RGBA → then downsample to the 2048-edge display cap). Parallel spawn can stack those peaks, but the dominant waste is **per image** — and users care more about images appearing quickly on open than about the most aggressive concurrency throttle. Steady-state budgets from `bound-preview-raster-memory` are already in place; this change cuts transient peak **without** making ordinary multi-image warm feel serialized.

## What Changes

- Restructure the raster decode path so oversized bitmaps are reduced toward the display edge **before** (or without) retaining a full-resolution RGBA copy — prefer consuming ownership (`into_rgba8` / resize-on-`DynamicImage`) over `to_rgba8` + post-hoc downsample. This is the primary peak lever and does not serialize appearance.
- Where the image crate allows cheaper subsampled decode for JPEG (and similarly cheap paths for other formats), use them when the source dimensions exceed the display edge, without changing the final display cap or visual contract for correctly decoded images.
- Keep a **high** in-flight safety cap (initial default: **8**) so pathological documents with dozens of images cannot stampede the pool, while typical docs still warm many images in parallel. Prefer parallel warm over a tight throttle; do not use a default of 2.
- When source dimensions can be probed cheaply before full decode, treat oversized ("heavy") images with a separate, tighter heavy-slot limit (initial default: **3**) so many small icons can still proceed in parallel while a few large photos do not each hold a full-res intermediate at once.
- Keep SVG rasterization at the existing display-edge clamp; SVG counts as a normal in-flight task, not automatically as heavy, unless its target pixmap would exceed the display edge (it already clamps).
- Extend diagnostics / harness notes so a post-change decode-spike probe can show a lower peak relative to the same workload, without gating CI on absolute bytes.

Non-goals: changing `PREVIEW_IMAGE_CACHE_MAX_BYTES`, claim/release semantics, or the 2048 display edge; mimalloc / global allocator swap; MathCache or DiagramCache budget changes; inactive-tab dormancy (already done); GPU atlas behaviour; making warm feel single-threaded for ordinary documents.

## Capabilities

### New Capabilities

None. Decode peak is part of the existing preview-image lifecycle.

### Modified Capabilities

- `preview-image-memory`: the capability today bounds *retained* decoded rasters (budget, eviction, display edge). It needs to additionally require that oversized sources do not retain a full-resolution RGBA intermediate solely to be downsampled afterward, and that any concurrency controls preserve parallel warm for typical documents (high overall cap; tighter limit only for probed heavy/oversized work).

## Impact

- `src/app/preview_image.rs` (`load_preview_image`, `ensure_preview_images`, downsample helpers) and unit/GPUI tests around decode size and in-flight accounting.
- A small in-flight / heavy-slot counter owned by `PreviewImageCache` or `MarkionApp`, using existing GPUI `background_spawn` — no new async runtime.
- `docs/memory-retention.md` decode-spike section and follow-up list; harness probe remains informational.
- Relies on `PreviewImageCache` from `bound-preview-raster-memory` and process counters from `report-process-memory-footprint` (both implemented; archive/sync of their specs may still be pending).
