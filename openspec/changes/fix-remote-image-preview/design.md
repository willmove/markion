## Context

All three image-bearing preview paths—Markdown/Read preview blocks, Visual Edit image blocks, and supported raw-HTML `<img>` parts—already call `preview_image_source` in `src/app/preview.rs`. The helper recognizes a URL containing `://` as remote and converts the original string into GPUI's `ImageSource`.

GPUI's `Application::new()` initializes the app context with `NullHttpClient`. Markion's bootstrap does not replace it, while GPUI's own remote-image example explicitly installs a Reqwest-backed client with `cx.set_http_client`. Consequently, every remote image load currently fails with `No HttpClient available`, including fragment-free URLs. This is the primary runtime failure.

The failing documents use valid Markdown destinations such as `https://mmbiz.qpic.cn/.../640?wx_fmt=png&from=appmsg#imgIndex=0`. GPUI 0.2.2 decides whether a string is a URI by parsing it as an HTTP request URI; that parser rejects URI fragments, so GPUI classifies the whole string as an embedded asset name instead of a network resource. The fragment is client-side metadata and must not be sent in an HTTP request.

The relevant data flow is:

`startup` → `shared TLS HTTP client in App` → `cached PreviewBlock / VisualBlock / HtmlPreviewPart URL` → `preview_image_source` → `GPUI ImageSource` → `GPUI async HTTP image loader`

The Markdown-derived blocks and their URLs remain cached per document version. This change affects only the final image request source and does not mutate or rederive cached document state.

## Goals / Non-Goals

**Goals:**

- Render supported HTTP(S) images whose Markdown destinations contain URI fragments.
- Make ordinary fragment-free HTTP(S) images load by providing GPUI with a real application HTTP client.
- Preserve all request-relevant URL components, especially query parameters used by image CDNs.
- Use one normalization path for Markdown preview, Read mode, Visual Edit, and supported raw-HTML images.
- Keep rendered image blocks visually clean by omitting redundant alt/URL/title chrome and excluding it from preview text selection.
- Preserve local image resolution, Markdown source, exports, and derived-state cache behavior.

**Non-Goals:**

- Implementing HTTP authentication, cookies, referer spoofing, retries, or workarounds for servers that reject ordinary image requests.
- Adding a persistent disk cache or a separate image download subsystem.
- Expanding the supported image formats beyond GPUI's existing loader.
- Rewriting fragment-bearing URLs in the document or exported output.

## Decisions

### Install one Reqwest-backed GPUI HTTP client during startup

Create a small adapter for GPUI's `HttpClient` trait using the same Zed-maintained Reqwest package already present in GPUI's dependency graph. The adapter owns one reusable Reqwest client and a single-worker Tokio runtime because GPUI's executor is not a Tokio runtime. Install it once through `cx.set_http_client` before windows or image elements are created. Preserve request methods, headers, redirect policy, status, response headers, and response bytes.

Alternative considered: download each image manually in preview rendering code. That would duplicate GPUI's image cache/decoder, introduce per-render networking state, and risk invalidating the document rendering invariants.

### Remove the fragment only from HTTP(S) request sources

Introduce a small request-URL normalization step at `preview_image_source`: for case-insensitive `http://` and `https://` sources, remove the first literal `#` and everything after it before constructing the GPUI URI image source. Keep the path and query byte-for-byte unchanged. Local paths and other source forms continue through their existing branches.

This follows URI semantics: fragments identify a client-side secondary resource and are not part of an HTTP request target. It also avoids adding a URL-parsing dependency for a single unambiguous operation.

Alternative considered: percent-encode `#` and keep the suffix. That would turn the fragment into part of the request path/query and request a different server resource.

### Normalize at the shared rendering boundary

Keep the source URL stored in `PreviewBlock`, `VisualBlockKind`, and `HtmlPreviewPart` unchanged, and normalize only when creating `ImageSource`. The existing call sites then receive the fix together, while source ranges, exports, and caches retain the exact authored URL.

Alternative considered: strip fragments while parsing Markdown. That would leak a rendering workaround into canonical derived data, alter displayed/copied metadata, and require duplicate handling for raw HTML.

### Keep successful rendered image blocks image-only

Remove the unconditional caption and URL/title children from Markdown preview and Visual Edit image panels. Also stop advertising image metadata as `PreviewTextRunId` entries, so Select All and multi-format copy reflect only text that is actually visible in the preview. The authored alt text, URL, and title remain in the source-backed block model for editing and export.

Alternative considered: hide only the URL while retaining `Image: <alt>`. The reported content uses generic alt text such as “图片”, so this would retain another redundant line and leave selection behavior inconsistent with an image-only preview.

### Test normalization separately from live networking

Add deterministic unit tests around request-source normalization covering a fragment-bearing WeChat-style HTTPS URL, ordinary HTTP(S) URLs, query preservation, and local paths containing `#`. Retain parser coverage that proves the authored URL reaches the image preview block unchanged. Do not make the test suite depend on an external CDN.

Alternative considered: an integration test that fetches the reported URL. That would be slow and flaky, and it would conflate URL classification with network availability and third-party policy.

## Risks / Trade-offs

- [A literal `#` intended as request data is removed] → Only literal fragment delimiters are removed; data intended for the server must be percent-encoded as `%23` according to URI syntax and remains untouched.
- [A server still refuses or cannot serve the image] → Keep GPUI's existing asynchronous loader; this change guarantees correct request construction, not third-party availability or a new failure-diagnostics UI.
- [The HTTP client cannot be initialized] → Log the startup error and keep the editor usable with local images; focused construction and loopback-request tests cover the normal path.
- [Normalization diverges across preview modes] → Keep all image call sites routed through `preview_image_source` and cover the shared helper rather than duplicating logic.
- [Rendering work invalidates Markdown caches] → Perform the operation after cached derived blocks are read; do not modify document versions, cache keys, or parsing.
