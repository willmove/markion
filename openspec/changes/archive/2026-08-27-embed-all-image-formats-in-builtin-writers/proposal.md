## Proposal

### Why

After the remote-image fix, two gaps remain in the built-in writers: the PDF writer still turns every remote and `data:` image into text, and both writers drop GIF/WebP/SVG payloads (DOCX rasterizes nothing, PDF only accepts SVG from local files). Documents built from web content routinely carry exactly these formats, and every dropped picture is an `alt: url` line in the exported file.

### What Changes

- A shared normalization step reduces every resolved image payload — local file, prefetched remote bytes, decoded `data:` URI — to one of the embeddable forms: PNG and JPEG pass through with sniffed dimensions, SVG passes through as vector text for the PDF writer (which already consumes it natively) and is rasterized to PNG for the DOCX writer, and other raster payloads (GIF, WebP, …) are decoded and re-encoded as PNG.
- The built-in PDF writer gains the same three resolvable sources as DOCX: local files (now without an extension allow-list — the payload is normalized by content), prefetched remote bytes, and decoded `data:` URIs. The export flow's prefetch now runs for PDF exports as well as DOCX-with-embed-policy, reusing the same bounded concurrent fetch.
- The DOCX writer embeds normalized payloads, so remote/local GIF, WebP, and SVG images now export as embedded PNGs (SVG rasterized at 2x supersampling with system fonts loaded, reported at natural size).
- Unresolvable or undecodable sources keep the `alt: url` text fallback on both writers; the export always succeeds.
- Dependency: the `image` crate gains the `gif` and `webp` decode features.

### Impact

- Affected spec: `export` — the PDF and DOCX image-embedding requirements are updated.
- Code: `Cargo.toml` (image features), `src/export.rs` (normalization helper, DOCX embed path, `build_pdf_ir`/`pdf_image_block` remote/data resolution), `src/lib.rs` (`build_pdf_ir` call site passes the prefetched map), `src/app/documents.rs` (prefetch condition includes PDF).
- Non-goals: animated-GIF frame sequencing (the first frame embeds); remote-image caching across exports (each export refetches, as today); the pandoc engine path (unchanged).
