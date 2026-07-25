## 1. Remove fair-share degradation

- [x] 1.1 Delete `fair_share_bytes` / `fair_share_max_edge` and the `max_edge` plumbing in `schedule_pending_preview_decodes`; decode via `load_preview_image` at `PREVIEW_IMAGE_MAX_EDGE`.
- [x] 1.2 Remove `shrink_claimed_ready_to_fair_share`; rework `fit_ready_under_budget` to: evict unclaimed LRU → store with overshoot up to a hard ceiling (2× budget) while claimed entries fill the budget → last-resort downscale the incoming raster to the remaining allowance.
- [x] 1.3 Rewrite the c89b7e4 tests that encode fair-share expectations (`byte_budget_keeps_pending_and_shrinks_when_only_claimed_ready_exist`, etc.) to assert: claimed entries keep their dimensions; overshoot ceiling honored; incoming-only downscale.

## 2. SVG supersampling

- [x] 2.1 Rasterize SVG at 2× (clamped) in `rasterize_svg_bytes`; extend `PreviewImageReady` with the intrinsic display size (or a supersample factor).
- [x] 2.2 In `preview_image_view`, present supersampled entries with an explicit width of the intrinsic size (pattern from `visual_diagram_editor`); raster images keep implicit sizing.
- [x] 2.3 Unit test: SVG ready entry reports raster dimensions ≈ 2× its display size.

## 3. Release demotes to LRU

- [x] 3.1 Change `PreviewImageCache::release`/`release_all` to keep zero-claim ready entries as unclaimed LRU; move `cx.drop_image` responsibility to actual eviction/removal paths (tab close paths still drop when eviction occurs).
- [x] 3.2 Update dormancy/tab-close GPUI tests: re-activation reuses ready entries (no pending placeholder); budget pressure still evicts unclaimed entries and drops GPUI images.
- [x] 3.3 Confirm memory report accounting (`completed_bytes`, ready counts) stays correct across demote/evict.

## 4. Verification

- [x] 4.1 `cargo fmt --check`, `cargo test --workspace` (app tests are in the bin target; pre-existing clippy failures unrelated).
- [ ] 4.2 Manual check with `examples/memory_fixtures/with_images.md` and an image-heavy document on a HiDPI display: sharp after load, no post-load degradation, no placeholder flash on tab switch.
- [x] 4.3 `openspec validate restore-preview-image-fidelity --strict`.
