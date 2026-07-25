# Restore preview image fidelity under memory budgets

## Why

Since the memory-retention work (cef4531 "lower decode peaks" and especially c89b7e4 "fair-share shrink"), preview images render blurry after loading. Three compounding causes, all in `src/app/preview_image.rs`:

1. **Pre-emptive fair-share decode cap** (c89b7e4): `schedule_pending_preview_decodes` decodes every image at `fair_share_max_edge(64 MiB, claimed_key_count)` — a cap derived from how many image *keys* are claimed, not from actual byte pressure or the images' real sizes. 8 claimed images → 1448 px cap; 16 → 1024 px; 30 → ~730 px. On a 2× (HiDPI) display a ~700-logical-px content column needs ~1400 device pixels, so any document with more than a handful of images decodes below display resolution and is upscaled at paint — blurry, unconditionally, even when the cache is nearly empty.
2. **On-screen shrinking** (c89b7e4): under actual budget pressure, `shrink_claimed_ready_to_fair_share` resamples already-displayed claimed images down to fair share. The user watches a sharp image turn blurry moments after it loads ("图片加载后变模糊"). Shrunk entries also never recover when pressure eases, and repeated shrinks resample already-resampled bitmaps, compounding the loss.
3. **SVG rasterized at 1×** (cef4531): the old gpui asset path rasterized SVG at `SMOOTH_SVG_SCALE_FACTOR = 2` and set `RenderImage::scale_factor = 2`, so SVGs were crisp on HiDPI. The new `rasterize_svg_bytes` renders at 1× and presents via `RenderImage` (whose `scale_factor` is `pub(crate)` — Markion cannot set it), so every SVG is upscaled on HiDPI. The diagram pipeline already solved this with `DIAGRAM_SUPERSAMPLE = 2.0` plus explicit presentation width; preview SVGs must do the same.

A related regression: `release()` removes a ready entry the moment its claim count reaches zero, so every tab switch drops the outgoing tab's decoded images and switching back re-decodes them all (placeholder flash + CPU churn), defeating the LRU cache.

## What Changes

- Decode at the full display cap: remove the fair-share edge from the decode path; always decode/rasterize toward `PREVIEW_IMAGE_MAX_EDGE` (2048).
- Never degrade claimed on-screen images. Under byte pressure: evict unclaimed LRU entries first; if the incoming raster still does not fit, downscale the *incoming* image only as a last resort, and re-decode from source at the target edge rather than resampling a resampled bitmap. Remove `shrink_claimed_ready_to_fair_share`.
- Rasterize preview SVGs with a 2× supersample (matching the diagram pipeline) and present them at their intrinsic (1×) size with an explicit width, so HiDPI displays get full pixel density.
- Keep zero-claim ready entries cached as unclaimed LRU (evictable under capacity/byte budget) instead of dropping them on release, so tab switches reuse decoded images.
- Keep: the 64 MiB completed budget, entry capacity, pending-never-evicted, claimed-never-evicted (anti-flicker), decode concurrency caps, and peak-lowering resize-before-RGBA order from `lower-preview-image-decode-peak`.

Non-goals: window-scale-factor-aware decode sizing (would need plumbing `Window` into the scheduler; 2048 covers 2× displays for typical columns); changing DiagramCache/MathCache; remote fetch behavior.

## Capabilities

### Modified Capabilities

- `preview-image-memory`: budget enforcement must not reduce the displayed resolution of claimed (on-screen) images; SVG sources must be presented at HiDPI-adequate pixel density; releasing the last claim demotes an entry to evictable instead of removing it.

## Impact

- `src/app/preview_image.rs`: remove fair-share plumbing (`fair_share_bytes`, `fair_share_max_edge`, `shrink_claimed_ready_to_fair_share`, `downscale_ready_to_max_bytes` resample-in-place), adjust `complete`/`release`, SVG supersampling, presentation sizing in `preview_image_view`.
- `src/app/preview.rs` image call sites unchanged (helper signature may gain intrinsic size).
- Unit tests in `preview_image.rs` covering budget behavior, plus the c89b7e4 tests that encoded fair-share expectations (rewritten).
