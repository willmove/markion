## Why

Network images are shown only as image metadata because Markion starts GPUI with its default null HTTP client, so every image fetch fails before reaching the server. Fragment-bearing destinations (for example WeChat image URLs ending in `#imgIndex=0`) additionally need request normalization because URI fragments are not valid HTTP request targets.

## What Changes

- Install one TLS-enabled HTTP client into the GPUI application context during startup so remote image resources can actually be fetched.
- Normalize remote image request URLs before handing them to GPUI by excluding the fragment while preserving the scheme, authority, path, and query string.
- Apply the same remote-image handling to Markdown preview, Visual Edit image blocks, and supported raw-HTML image previews.
- Render successful image blocks as images only, without appending the authored alt text, URL, or optional title as redundant visible/selectable metadata below them.
- Add regression coverage for fragment-bearing remote image URLs and for preservation of query parameters and local-image resolution.
- Non-goals: adding a general download manager, persisting remote images on disk, bypassing server authentication or anti-hotlink policies, or changing Markdown source/export output.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Require supported HTTP(S) Markdown images, including URLs with client-side fragments, to render cleanly in preview surfaces without altering the document source or exposing redundant URL metadata.

## Impact

- Affected code: application HTTP-client initialization, shared preview image-source resolution in `src/app/preview.rs`, and focused tests in the root app crate.
- The source Markdown and its fragment remain unchanged; only the URL used for the network request is normalized.
- Add direct root-crate dependencies on the Reqwest implementation already used by GPUI's HTTP stack and on the existing workspace Tokio runtime.
- Per-document-version derived Markdown caches remain unchanged; URL normalization occurs at the existing image-source boundary and does not introduce reparsing.
