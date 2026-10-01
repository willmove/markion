## Why

Markdown documents routinely embed images as base64 data URIs (`![](data:image/png;base64,...)`), a pattern produced by many editors, browsers, and export tools. Markion parses these URLs and passes them through to the preview-image loader, but the loader only knows how to read local files or fetch `http(s)://` URLs — so every data-URI image renders as a red missing-image placeholder instead of the picture. Users opening such documents see broken images for content that is fully self-contained in the source.

## What Changes

- Decode `data:` URI image destinations inline (parse MIME, base64-decode the payload) and feed the resulting bytes into the existing SVG/raster decode pipeline — no network fetch, no disk file required.
- Route data-URI images through a dedicated load branch so the shared HTTP client is never asked to resolve a `data:` scheme (today this is the exact failure point).
- Treat data-URI images as cacheable preview entries (keyed by their full URI) so repeated occurrences dedupe under the existing bounded LRU cache, consistent with local and remote images.
- Preserve current behavior for malformed data URIs: surface the explicit missing-resource placeholder with the raw URI, rather than silently dropping the image.
- Apply uniformly to preview, Visual Edit, and raw-HTML `<img src="data:...">` since all three funnel through the same loader seam.

## Non-goals

- No change to how data URIs are *authored* or *imported* — paste/drop continues to store managed resources on disk under the document's asset directory.
- No re-encoding or normalization of data URIs in the document source; the bytes are decoded for display only.
- No outbound fetch of any kind; data URIs are resolved entirely in-process.

## Capabilities

### New Capabilities

_(None.)_

### Modified Capabilities

- `document-resources`: Adds a requirement that data-URI image destinations be resolved and rendered inline (today the capability only covers local-file resources and explicit missing-resource recovery; data URIs currently fall through to the missing-resource state).

## Impact

- **Code:** `src/app/preview_image.rs` (`load_preview_image`, `PreviewImageKey` identity/accessors), `src/app/preview.rs` (`is_remote_resource` classification and identity prefixing). The decode → RGBA → BGRA → `RenderImage` path is reused unchanged.
- **Dependencies:** Add a direct dependency on the `data-url` crate (already present transitively via GPUI in `Cargo.lock`) for spec-compliant data-URI parsing. Base64 decoding is already available via the `base64` crate (transitive); the `data-url` crate exposes decoded bytes directly.
- **Caching/memory:** Data-URI images are accounted under the existing `PreviewImageCache` byte budget (64 MiB) and LRU eviction; no new unbounded retention. Large data URIs are classified as light for decode-concurrency purposes (unchanged from today's remote classification), which is acceptable.
- **Invariants preserved:** Per-document-version derived-state caches are untouched (image identity is keyed by URL, not version). The cached-text-handle and syntax-highlighting memoization are not affected.
