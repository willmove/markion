## Context

Markdown preview images currently become `ImageSource::Resource` paths/URLs via `preview_image_source`. GPUI loads them through `Window::use_asset` into `App::loading_assets`, which never evicts. Markion never calls `App::remove_asset` or `drop_image`. Diagrams already use a Markion-owned `DiagramCache` with capacity 128, but eviction is entry-count FIFO for completed keys only — a few large 2×-supersampled rasters can still retain hundreds of megabytes.

Memory diagnostics (`docs/memory-retention.md`) label `global.gpui_image_assets` as external and unenumerable. This change makes Markion the owner of decoded preview-image presentation data so that site becomes accounted and bounded.

## Goals / Non-Goals

**Goals:**
- Bound decoded Markdown preview-image memory with a completed-byte budget and entry/LRU eviction.
- Release image assets tied to a tab when that tab closes or its document is replaced.
- Avoid retaining full-resolution bitmaps for preview when a smaller decode suffices for display.
- Bound `DiagramCache` completed raster bytes similarly to `MathCache`.
- Keep image/diagram presentation correct: same URL still shows the same image; theme/diagram keys unchanged; no document mutation.

**Non-Goals:**
- Changing Mermaid sanitization, backend registry, or supersample factor.
- Evicting inactive-tab Markdown derived caches (`evict-inactive-tab-caches`).
- Perfect GPU atlas accounting (CPU-side `RenderImage` / asset drop is the contract; atlas cleanup follows GPUI's `drop_image`).

## Decisions

### Own preview images through a Markion cache that feeds `ImageSource::Render`

Rather than wrapping GPUI's retain-all resource loader and hoping eviction works, Markion loads/decodes images on a background task into `Arc<RenderImage>`, stores them in an app-level `PreviewImageCache`, and presents via `ImageSource::Render`. That path bypasses `loading_assets` retention for those sources.

Alternatives considered:
- **`RetainAllImageCache` + manual `remove`/`clear`**: still retain-all by default; eviction API exists but we would reimplement budgeting around it anyway.
- **Only call `remove_asset` on tab close without a Markion cache**: incomplete — assets remain for every unique URL ever shown until close, and there is still no byte budget while tabs stay open.

### Byte budget + LRU among ready entries; pending never evicted

Mirror `MathCache`:
- Cap completed bytes (initial default: 64 MB for preview images; 32 MB for diagram completed rasters — both configurable constants next to the caches).
- Cap entry count as a secondary guard.
- Pending entries are not evicted (same diagram invariant); if capacity is full of pending work, skip scheduling new work until a slot frees.

### Display-oriented decode limit for preview images

When decoding, clamp the longer edge to a maximum (e.g. 2048 device pixels) before producing the `RenderImage`, preserving aspect ratio. Preview already uses `.max_w_full()`; users do not need print-resolution bitmaps in the editor. Diagrams keep their existing supersample policy (sharpness requirement in `diagram-rendering`).

Remote images continue to use the existing HTTP client; decode happens after bytes arrive.

### Eviction triggers

1. Tab close / `replace_active_tab` / last-tab reset: drop cache entries whose only remaining referrer was that document's image URL set (refcount or per-tab claimed URL set).
2. Budget pressure: LRU among ready entries globally.
3. Explicit theme? Not required for ordinary photos; remote/local image keys do not include theme.

Per-tab claim set: when a tab syncs preview/visual blocks, it registers the image URLs it currently references. Closing the tab releases those claims; an entry with zero claims becomes immediately evictable even before LRU.

### Diagram byte budget is additive to entry capacity

Keep `DIAGRAM_CACHE_CAPACITY = 128` and add `DIAGRAM_CACHE_MAX_BYTES` (default 32 MiB). On `complete`, if adding the raster would exceed the byte budget, evict oldest completed entries until it fits or reject as too large (same pattern as math's `OutputTooLarge` for a single raster exceeding the whole budget).

### Diagnostics

`global.gpui_image_assets` becomes `global.preview_image_cache` (owned) with entry/pending/ready/bytes/budget counters. Diagram site gains `completed_bytes` / `budget_bytes`. Update `docs/memory-retention.md` after implementation with a fresh harness dump.

## Risks / Trade-offs

**Visible quality drop after downsampling** → Cap is high enough for retina preview columns; document that export/PNG paths are unaffected (export does not use this cache).

**Flicker when an LRU-evicted image scrolls back into view** → Accept re-decode; same as Chromium image cache behavior. Prefer claiming visible URLs so actively shown images are last to leave.

**Pending-full deadlock** → Same as today's diagram cache; document and keep the "retry next frame" behavior.

**Double memory during migration if both GPUI asset table and Markion cache hold the same image** → All Markdown preview `img()` call sites for `preview_image_source` MUST switch to the Render path; audit Visual Edit + HTML preview parts.
