## Why

Raw HTML images inside centered paragraph wrappers are parsed with the correct center alignment, but the shared preview renderer gives the image wrapper only its intrinsic width. As a result, the centering flex rule has no wider container to distribute space in, so the image remains at the left edge in Visual Edit, Read, and Split Preview.

## What Changes

- Make the shared raw-HTML image presentation wrapper fill the rendered content width before applying left, center, or right alignment.
- Add a regression test covering the supplied `<p align="center"><img ...></p>` shape and the shared renderer path.
- Preserve authored image dimensions, image loading/cache behavior, source mapping, and existing non-centered alignment behavior.

Non-goals: changing Markdown image alignment, the document-width preference, image sizing rules, or HTML parsing semantics.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. The existing `markdown-editing` capability already requires centered standalone HTML image blocks; this change restores that existing contract without changing the requirement.

## Impact

- `src/app/preview.rs`: shared HTML preview block layout for image parts.
- `src/parse.rs` or the relevant application test module: regression coverage.
- No new dependencies, public APIs, document-cache changes, or source/document mutations. The fix is presentation-only and preserves per-version derived Markdown caches.
