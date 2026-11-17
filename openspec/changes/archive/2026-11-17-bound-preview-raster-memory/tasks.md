## 1. Preview image cache core

- [x] 1.1 Add `PreviewImageCache` (pending/ready/error, entry cap, completed-byte budget, LRU order) in `src/app/`, with claim/release APIs keyed by normalized local path or remote request URL.
- [x] 1.2 Implement background fetch/decode that produces `Arc<RenderImage>`, applying the display-edge downsampling limit before the ready entry is stored.
- [x] 1.3 Wire completion to `cx.notify` without mutating document text, version, or derived Markdown caches; drop unclaimed late completions.
- [x] 1.4 Add unit tests: reuse of identical keys, pending dedupe, byte-budget eviction of ready entries, pending non-eviction, reject single raster larger than the budget.

## 2. Preview presentation integration

- [x] 2.1 Replace `img(preview_image_source(...))` call sites in Split/Read/Visual Edit/HTML preview with a helper that reads `PreviewImageCache` and emits `ImageSource::Render` or a pending/error placeholder.
- [x] 2.2 On `sync_preview_list` / `sync_visual_list`, refresh the active tab's image URL claims from populated blocks (including HTML `<img>` srcs).
- [x] 2.3 On tab close, `replace_active_tab`, and `reset_preview_list`, release that tab's claims and evict zero-claim ready entries, calling GPUI `drop_image` for removed rasters.
- [x] 2.4 Add GPUI tests: two tabs sharing one image keep it after one closes; unique images are gone from the Markion cache after their only tab closes; document version is unchanged across load/complete.

## 3. Diagram byte budget

- [x] 3.1 Add `DIAGRAM_CACHE_MAX_BYTES` and track completed raster bytes on `DiagramCache`, evicting oldest ready entries on `complete` when over budget.
- [x] 3.2 Reject a single diagram raster larger than the budget with a typed/render-failed path consistent with existing error presentation.
- [x] 3.3 Extend diagram cache unit tests for byte-budget eviction and oversized rejection; pending entries still never evicted.

## 4. Diagnostics and docs

- [x] 4.1 Update `memory_report` so preview images are an owned site (`global.preview_image_cache`) with entries/bytes/budget counters; keep any remaining unowned GPUI asset note only if still accurate.
- [x] 4.2 Update diagram memory accounting to expose `completed_bytes` and budget.
- [x] 4.3 Refresh `docs/memory-retention.md` with a post-change harness dump for `with_images` / `with_diagrams` and note the new budgets.
- [x] 4.4 Run `cargo fmt --check`, `cargo test --workspace`, and `openspec validate bound-preview-raster-memory`.
