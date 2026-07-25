## 1. Decode path peak reduction (primary)

- [x] 1.1 Replace `to_rgba8()` + post-hoc `downsample_rgba` with resize-on-`DynamicImage` (when over the display edge) followed by consuming `into_rgba8()` for non-SVG sources.
- [x] 1.2 Keep the retained longer-edge clamp identical to `PREVIEW_IMAGE_MAX_EDGE`; preserve BGRA swap and `RenderImage` construction.
- [x] 1.3 Opportunistically use cheaper subsampled JPEG (or other format) decode when dimensions exceed the display edge and the `image` API allows it; fall back to decode→resize→`into_rgba8` otherwise.
- [x] 1.4 Leave SVG rasterization structure as-is (already targets display-sized pixmap).
- [x] 1.5 Unit tests: oversized synthetic image yields ready dimensions ≤ max edge; already-small image keeps dimensions; unsupported identities still error cleanly.

## 2. Parallel-friendly concurrency safety valve

- [x] 2.1 Add `PREVIEW_IMAGE_DECODE_CONCURRENCY` (initial default **8**) and `PREVIEW_IMAGE_HEAVY_DECODE_CONCURRENCY` (initial default **3**) next to the preview-image cache limits.
- [x] 2.2 Track overall in-flight and heavy in-flight counts; `ensure_preview_images` must not exceed either applicable limit.
- [x] 2.3 Classify heavy only when a cheap size probe shows longer edge > display max; unclassified or small images count only against the overall cap (favor parallel warm).
- [x] 2.4 On decode completion, decrement counters and ensure remaining pendings can start without a user edit (notify / kick ensure); no document mutation.
- [x] 2.5 Tests: several small missing images may run concurrently up to the overall cap; more probed oversized images than the heavy limit never exceed heavy in-flight; stranded pendings do not remain after completions + notify.

## 3. Diagnostics and docs

- [x] 3.1 Re-run `memory_decode_spike_footprint_probe` (and, if useful, a multi-image warm note) and refresh the decode-spike section in `docs/memory-retention.md` with post-change figures and the new concurrency defaults.
- [x] 3.2 Update the follow-up candidates list to mark decode-peak work done / remaining (allocator swap still deferred).

## 4. Verification

- [x] 4.1 Run `cargo fmt --check` and ensure the touched module is clippy-clean relative to existing project noise.
- [x] 4.2 Run `cargo test --workspace`.
- [x] 4.3 Run `openspec validate lower-preview-image-decode-peak`.
