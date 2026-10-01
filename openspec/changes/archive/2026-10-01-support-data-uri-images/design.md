## Context

See `proposal.md` (Why) for motivation. Today all preview images — Markdown `![](...)`, Visual Edit `Image` blocks, and raw-HTML `<img>` — funnel through one loader seam in `src/app/preview_image.rs`:

- `PreviewImageKey::from_url` classifies a URL as either `local:<canonical-path>` (via `is_remote_resource` → false) or `remote:<url>` (via `is_remote_resource` → true, which already returns true for any `data:` prefix).
- `load_preview_image(key)` then takes exactly one of two branches: `key.local_path()` → `std::fs::read`, or `key.remote_url()` → `network::fetch_url_bytes` (a `reqwest` GET). A `data:` URI takes the `remote_url()` branch, hands `data:image/png;base64,...` to `reqwest`, and the request fails — surfacing the red missing-image placeholder.

The downstream path (SVG rasterize → RGBA → BGRA swap → `RenderImage`, all in `preview_image.rs`) is format-agnostic and already byte-oriented. The decode, cache, claim, byte-budget, and LRU machinery is keyed by `PreviewImageKey::identity` and needs no structural change. The seam to fix is therefore narrow: get bytes for a `data:` URI without going through the HTTP client, and make the key expose that variant.

The `data-url` crate (RFC 2397) is already a transitive dependency in `Cargo.lock` (pulled in by GPUI). Its `DataUrl::process(url).decode()` API returns the percent-decoded / base64-decoded body plus the MIME type in one call, which covers both the `;base64` and URL-encoded forms the spec requires.

Data flow (rendering + caching touch points — unchanged except the boxed step):

```
preview/visual/html blocks (per document version, Arc-cached — UNCHANGED)
  └─ collect_preview_image_urls ─► PreviewImageKey::from_url
        └─ identity = "data:<full-uri>"        ← NEW prefix (was "remote:")
  └─ PreviewImageCache (LRU + byte budget — UNCHANGED)
        └─ load_preview_image(key)
              ├─ key.local_path()  → fs::read          (UNCHANGED)
              ├─ key.remote_url()  → fetch_url_bytes   (UNCHANGED, http(s) only)
              └─ key.data_url()    → DataUrl::process  ← NEW BRANCH
                    └─ bytes ─► rasterize_svg_bytes / decode_raster_bytes (UNCHANGED)
              └─ rgba_to_ready ─► PreviewImageEntry::Ready (UNCHANGED)
```

## Goals / Non-Goals

**Goals:**
- Render base64 **and** URL-encoded `data:` URI images of every format the file-based path already supports (PNG/JPEG/GIF/WebP/BMP/TIFF/SVG), through preview, Visual Edit, and raw HTML.
- Keep the fix inside the existing loader seam so parsing, block types, the cache, claims, and GPUI rendering are untouched.
- Make data-URI images dedupe under the existing cache and obey the existing byte budget.
- Preserve the explicit missing-resource state for malformed data URIs (no silent drops).

**Non-Goals:**
- Do not change authoring/import (paste/drop still writes managed resources to disk).
- Do not re-encode, shorten, or otherwise mutate data URIs in the document source.
- Do not add size-based special-casing for "large" data URIs beyond what the existing byte budget + LRU already enforce.

## Decisions

### Decision 1 — Use a dedicated `data:` identity prefix instead of `remote:`

Today `PreviewImageKey::from_url` prefixes all non-local URLs with `remote:`. Data URIs would otherwise inherit `remote:data:...`, which then needs string sniffing inside `remote_url()` to avoid handing a `data:` string to `reqwest`. Cleaner: branch in `from_url` so a `data:` URL gets identity prefix `data:` (the bare URI), and add a `data_url()` accessor returning `Option<&str>` alongside `local_path()` / `remote_url()`.

- **Why:** keeps each accessor single-purpose; `remote_url()` keeps meaning "feed this to reqwest", so the HTTP client can never receive a `data:` scheme. The identity string is still self-describing and recoverable.
- **Alternative considered:** sniff `remote:`-prefixed identities for a leading `data:` inside `load_preview_image`. Rejected — muddies `remote_url()`'s contract and re-breaks if anyone else reads the identity.

### Decision 2 — Decode via the `data-url` crate

`DataUrl::process(url)?.decode()` returns `(Vec<u8>, Option<Fragment>)`, handling both `;base64` and the percent-encoded form, plus malformed-input detection (it returns `Err` for truncated base64, bad percent-escapes, etc.). Use it directly; do not hand-roll base64/percent decoding.

- **Why:** spec-compliant (RFC 2397), already vendored transitively (no new download), and its `Err` variants map cleanly onto the existing `load_preview_image → Result<_, String>` error path that produces the missing-resource placeholder.
- **Alternative considered:** pull in `base64` directly and split the URI by hand. Rejected — re-implements the MIME-type + `;base64` grammar the spec already requires and diverges on edge cases (URL-encoded payloads, parameter ordering, fragments).

### Decision 3 — Route on the decoded bytes for SVG detection

After decoding, feed the bytes into the **existing** `is_svg` / `rasterize_svg_bytes` / `decode_raster_bytes` block at the tail of `load_preview_image` (`preview_image.rs:413-433`). The current `is_svg` heuristic already falls back to scanning the first bytes for `<svg`, which works for data URIs; extend it to also recognize `image/svg+xml` from the data URL's MIME when the byte scan is inconclusive. No new decode path.

### Decision 4 — Treat data-URI loads as "light" for concurrency

`probe_is_heavy` (`preview_image.rs:388-402`) only probes local files (returns `false` otherwise). Data URIs therefore classify as light, the same as today's remote images. This is acceptable: the existing overall cap (8 in-flight) bounds parallelism, and the byte budget bounds retained memory. No change.

## Risks / Trade-offs

- **[Very large inline data URIs inflate `.md` files and decode cost]** → Mitigated by the existing 64 MiB retained-byte budget and LRU eviction; decode is one-shot per cache residency and runs on the background thread pool already used for remote fetches. No new bound introduced; behavior matches how a similarly-sized remote image is treated.
- **[`data-url`'s `decode()` copies the full body into a new `Vec`]** → Acceptable: the bytes are about to be handed to the decoder anyway, which also copies. A future optimization could decode in place, but is not worth the complexity now.
- **[Malicious / malformed data URI could be expensive to parse]** → `data-url` fails fast on invalid base64/percent-encoding (returns `Err`), surfacing the missing-resource placeholder. No unbounded work; the existing overall decode cap still applies.
- **[Identity prefix change (`remote:` → `data:`) invalidates any in-flight cache entries]** → No migration concern: the cache is in-memory and per-session, and data URIs currently never reach the `Ready` state anyway (they always error out), so there is nothing valid to invalidate.

## Migration Plan

None. In-memory cache only; no persisted state, no file format change, no config. Building and running the editor with the change is the whole rollout. Rollback = revert the commit.

## Open Questions

None.
