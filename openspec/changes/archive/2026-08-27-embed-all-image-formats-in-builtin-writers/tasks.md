## Tasks

### 1. Shared normalization

- [x] 1.1 Add the `gif` and `webp` decode features to the `image` dependency
- [x] 1.2 Add a normalization helper in `src/export.rs`: sniff PNG/JPEG (magic bytes + dimensions), detect SVG payloads by content, and decode-plus-re-encode every other decodable raster payload to in-memory PNG; undecodable input returns `None`

### 2. DOCX writer

- [x] 2.1 Route `embed_image` through the normalizer: PNG/JPEG pass through, GIF/WebP embed as re-encoded PNG, SVG rasterizes to PNG (usvg with a system-font database, 2x supersample, dimensions reported at natural size)
- [x] 2.2 Tests: a GIF payload in the prefetch map and an SVG payload in the prefetch map both embed as PNG `w:drawing` parts

### 3. PDF writer

- [x] 3.1 Thread the prefetched remote map through `build_pdf_ir` into `pdf_image_block` (Markdown and raw-HTML image paths) and resolve image bytes from the same three sources as DOCX instead of the local-extension allow-list
- [x] 3.2 Map normalized payloads onto `PdfImageData`: PNG/JPEG bytes pass through with sniffed dimensions, SVG passes through as the native vector variant, GIF/WebP embed as re-encoded PNG; unresolvable sources keep the text fallback
- [x] 3.3 Tests: prefetched remote PNG and SVG produce `PdfBlock::Image` entries (raster and vector variants), a `data:` URI embeds, and a GIF payload normalizes to PNG

### 4. App-layer prefetch scope

- [x] 4.1 Extend the export prefetch condition to PDF exports (PDF has no image policy — it always embeds) in `src/app/documents.rs`, reusing the same status line and map

### 5. Verification

- [x] 5.1 Run `cargo test --workspace`, `cargo fmt`, clippy on the touched files, and `openspec validate embed-all-image-formats-in-builtin-writers`
