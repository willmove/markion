## Tasks

### 1. Document layer

- [x] 1.1 Add `MarkdownDocument::remote_image_urls()` collecting deduplicated `http(s)` image URLs from preview blocks (Markdown images, raw-HTML `<img>`, blockquote-nested images) with unit coverage

### 2. Built-in DOCX writer

- [x] 2.1 Extend `DocxRenderState` with a `remote_images: HashMap<String, Vec<u8>>` map and route `embed_image` through it: remote URLs resolve from the map, `data:` URIs decode inline, local paths read from disk as today
- [x] 2.2 Thread the map through `build_docx_bytes` / `write_docx` / `render_docx_document_xml` and `export_to_with` (the `export_to` convenience wrapper passes an empty map), updating all call sites and tests
- [x] 2.3 Writer tests: prefetched PNG embeds as `w:drawing` + `word/media` entry; a missing map entry and a non-PNG/JPEG payload keep the `alt: url` text fallback; a base64 PNG `data:` URI embeds

### 3. App-layer prefetch

- [x] 3.1 Add a bounded concurrent fetch helper to `src/app/network.rs` (shared client, per-URL timeout and size cap, failures logged and omitted) with loopback-server coverage
- [x] 3.2 In `export_with_prompt`, snapshot the document clone and preferences, prefetch remote images on the background executor when exporting DOCX with the embed policy, and run the export from the snapshot so slow downloads cannot export the wrong tab

### 4. Localization and spec

- [x] 4.1 Retitle the image-policy option from "Embed local images" to "Embed images" in all seven languages and add the prefetching status line
- [x] 4.2 Run `cargo test --workspace` and `openspec validate embed-remote-images-in-docx-export`
