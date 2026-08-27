## Proposal

### Why

The built-in DOCX writer — now the default export backend — turns every remote (`http(s)`) and `data:` image into an `alt: url` text paragraph. Documents assembled from web content or pasted screenshots (data URIs) therefore lose all their pictures when exported to Word, while the pandoc engine path downloads and embeds remote images itself. Users switching to the dependency-free backend understandably read this as an export bug.

### What Changes

- The built-in DOCX writer embeds remote images when their bytes are available: the export flow prefetches every remote image referenced by the document (Markdown image syntax and raw-HTML `<img>`, deduplicated) concurrently on a background thread with bounded timeouts and size caps, then hands the fetched bytes to the writer.
- A remote image that fails to fetch (offline, HTTP error, oversized, non-PNG/JPEG payload) keeps the existing `alt: url` text fallback — the export always succeeds, exactly like a missing local file today.
- `data:` URIs decode inline (no network) and embed when the payload is PNG/JPEG.
- The prefetch only runs for DOCX export with the `embed` image policy; the `text-fallback` policy still exports every image as text on both backends.
- The Preferences panel image-policy label drops the "local" qualifier ("Embed images") to match the new behavior.

### Impact

- Affected spec: `export` — the "Built-in DOCX fallback embeds local images" requirement gains remote/data-URI embedding with failure-tolerant fallback.
- Code: `src/lib.rs` (`remote_image_urls` collector; `export_to_with` takes the prefetched remote-image map), `src/export.rs` (embed from fetched bytes / decoded data URIs), `src/app/documents.rs` (background prefetch before the export update), `src/app/network.rs` (bounded concurrent fetch helper), `src/i18n.rs` (one label per language).
- Non-goals: the built-in PDF writer keeps its current remote-image text fallback (future candidate); the pandoc engine path is unchanged (pandoc already fetches remote images); non-raster remote formats (WebP/GIF/SVG) still fall back to text.
