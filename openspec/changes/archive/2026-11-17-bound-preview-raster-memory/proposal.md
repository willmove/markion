## Why

Memory diagnostics showed that preview Markdown images are decoded into GPUI's process-global asset table with no eviction, and that `DiagramCache` caps entry count but not raster bytes. Closing a tab or leaving an image off-screen does not release those bitmaps, so multi-tab sessions with ordinary photos or screenshots can exceed Obsidian's resident memory despite Markion's smaller baseline. This change bounds that raster layer without changing Markdown semantics or Visual Edit editing behavior.

## What Changes

- Introduce a Markion-owned, byte-budgeted image cache for Markdown `![]()` / HTML `<img>` preview sources, instead of relying solely on GPUI's unbounded `App::loading_assets` retain-all path.
- Evict or drop decoded image assets when tabs close, documents are replaced, or the image budget is exceeded (LRU among completed entries).
- Decode or present local/remote preview images at a display-oriented maximum edge (downsampling oversized sources) so a 4000×3000 photo does not retain a full-resolution BGRA buffer when the preview column is a few hundred pixels wide.
- Add a completed-raster byte budget to `DiagramCache`, matching the existing MathCache pattern, so a small number of large diagrams cannot retain unbounded memory.
- Extend memory diagnostics sites so image and diagram budgets report budget usage (entries, bytes, evictions) rather than only external/unbounded labels.

Non-goals: inactive-tab derived Markdown cache eviction (separate change `evict-inactive-tab-caches`); lowering MathCache's 128 MB budget; highlight-cache LRU; viewport-limited editor `shape_text`.

## Capabilities

### New Capabilities
- `preview-image-memory`: bounded lifecycle for decoded Markdown preview images (budget, eviction on tab close / pressure, display-oriented decode limits).

### Modified Capabilities
- `diagram-rendering`: diagram presentation cache MUST bound completed raster bytes in addition to entry count; pending entries remain non-evictable as today.

## Impact

- Preview image construction in `src/app/preview.rs` and any Visual Edit image presentation paths that use `preview_image_source`.
- New or extended cache module beside `src/app/diagram.rs` / `src/app/network.rs`; uses GPUI `remove_asset` / `drop_image` / optional `ImageCache` APIs already present in gpui 0.2.2.
- `DiagramCache` eviction policy in `src/app/diagram.rs`.
- `src/app/memory.rs` report sites and `docs/memory-retention.md` follow-up notes.
- Relies on the in-tree memory diagnostics from `add-memory-diagnostics` (implemented; may still be awaiting archive).
