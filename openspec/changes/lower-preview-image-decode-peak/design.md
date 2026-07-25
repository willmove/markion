## Context

`PreviewImageCache` already owns ready rasters, claims them per tab, budgets completed bytes (64 MiB), and clamps the retained longer edge to 2048. Closing a tab releases claims and calls `drop_image`. That is the steady-state contract from `bound-preview-raster-memory`.

The remaining problem is **transient** memory during `load_preview_image`:

```
  ensure_preview_images
       │
       ├─ for each missing key: spawn background task
       │
       └─ load_preview_image
              read bytes
              image::load_from_memory → DynamicImage   ← full resolution
              .to_rgba8()                              ← copy, still full res
              downsample_rgba → 2048 edge              ← only now shrinks
              BGRA swap → RenderImage
```

For a 4000×3000 photo the intermediates dominate the peak. Parallel spawn can stack peaks, but product priority is **fast parallel appearance** ("打开就很快出齐图"). Therefore peak reduction must come mainly from shrinking **per-image** intermediates, with concurrency used only as a safety valve — not as a warm throttle of 2.

## Goals / Non-Goals

**Goals:**

- Lower peak process memory while opening/warming image-heavy documents.
- Preserve parallel warm latency for typical multi-image documents (many images in flight; not serialized).
- Keep the retained ready bitmap at the same display edge and quality class as today.
- Preserve pending / ready / error semantics, claim/release, and off-frame loading.
- Make the improvement observable via the existing decode-spike / footprint diagnostics (informational, not a CI byte gate).

**Non-Goals:**

- Changing cache budgets, entry capacity, or the 2048 display edge constant's meaning for retained rasters.
- Allocator swap (mimalloc) — separate follow-up if peak falls but after-drop current stays high.
- Changing remote fetch policy, SVG fidelity beyond today's edge clamp, or export/PNG paths (they do not use this cache).
- Perfect zero-copy decode for every format — prefer large, format-agnostic wins first.
- Minimizing concurrency at the expense of warm UX (rejected: default-2 throttle).

## Decisions

### Primary lever: resize before (or instead of) full RGBA materialization

Change the raster path for non-SVG images to:

1. Decode to `DynamicImage` (unavoidable with the current `image` API for most formats).
2. If longer edge > `PREVIEW_IMAGE_MAX_EDGE`, **resize the `DynamicImage`** to the display size **before** producing RGBA.
3. Convert to RGBA with **`into_rgba8()`** (consume) rather than `to_rgba8()` (clone).
4. Proceed with BGRA swap as today.

This eliminates the simultaneous full-res `DynamicImage` + full-res RGBA pair. It lowers each task's peak **without** forcing tasks to wait on each other.

Alternatives considered:

- **Only switch to `into_rgba8` without reorder** — saves one copy but still holds full-res RGBA before downsample; insufficient alone.
- **Tight concurrency (limit 2) as the main fix** — rejected after product feedback; slows "出齐图" too much for ordinary docs.

### JPEG / opportunistic subsampled decode

When dimensions are known (or cheaply probed) and exceed the display edge, prefer a decode path that never materializes full resolution if the `image` crate + enabled features allow it (e.g. JPEG scale factors). If probing fails or the format has no cheap path, fall back to decode → resize-on-`DynamicImage` → `into_rgba8`. Correctness and the final display edge trump peak savings.

### Concurrency: high overall cap + optional heavy-slot limit

Two constants next to the cache:

| Constant | Initial default | Role |
|----------|----------------:|------|
| `PREVIEW_IMAGE_DECODE_CONCURRENCY` | **8** | Hard ceiling on all in-flight fetch/decode tasks (safety valve for pathological docs) |
| `PREVIEW_IMAGE_HEAVY_DECODE_CONCURRENCY` | **3** | Tighter ceiling for tasks classified as heavy |

**Heavy** means: a cheap probe (headers / `ImageReader` size) shows longer edge > `PREVIEW_IMAGE_MAX_EDGE`, or probe failed and the format is typically large (optional heuristic — prefer probe success). Small icons, badges, and already-display-sized assets do **not** consume heavy slots; they only count against the overall cap of 8.

Scheduling rules:

- Keys that cannot start stay `Pending` and retry on the next ensure pass after a completion `notify` (same spirit as pending-full diagram work).
- Prefer "rely on next ensure after notify" unless tests show stranded pendings; then kick ensure on completion.
- If size cannot be probed without a full decode, start under the overall cap and treat as non-heavy (favor appearance); the resize-before-RGBA path still bounds that task's peak once decode begins.

Alternatives considered:

- **Unbounded spawn** — rejected as a safety valve for 50+ image docs.
- **Default 2 for all images** — rejected; hurts parallel warm.
- **Overall 8 only, no heavy class** — acceptable fallback if probing is too flaky; implement heavy slots when probe is reliable, otherwise document that only the overall cap applies.

### SVG path unchanged in structure

SVG already rasterizes into a pixmap sized to the display edge. It counts against the overall in-flight cap. It does not need the heavy class unless implementation later treats large SVG targets specially (not required now).

### Diagnostics stay observational

Update `docs/memory-retention.md` and re-run `memory_decode_spike_footprint_probe`. Tests assert structural properties (overall in-flight ≤ 8, heavy in-flight ≤ 3 when classification applies, retained edge ≤ max) and informational peak commentary — never absolute RSS thresholds in CI.

## Risks / Trade-offs

**Peak still stacks when many large photos warm under heavy-slot = 3** → Accepted: product prefers faster appearance; per-image path still cuts each photo's intermediate from ~full-res×2 down toward display-sized. Pathological galleries may still spike; safety cap 8 bounds the worst case better than today.

**Probe misclassification (large file marked light)** → Falls back to overall cap + resize-before-RGBA; peak higher than ideal for that task but still better than `to_rgba8` + post-downsample.

**Resize-before-RGBA quality slightly different from resize-after-RGBA** → Negligible at preview scale; unit-test final dimensions against the edge clamp.

**JPEG scale factors are coarse (1/2, 1/4, 1/8)** → May undershoot then upscale slightly — still far cheaper than full-res RGBA. Best-effort only.

**Retry-via-next-ensure could starve** → Completions already `cx.notify()`; add an explicit kick if a test shows stranded pendings.
