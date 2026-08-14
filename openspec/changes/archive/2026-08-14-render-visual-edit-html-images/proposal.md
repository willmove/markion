# Proposal: render-visual-edit-html-images

## Why

Visual Edit collapses any prose block containing raw HTML (`<img …>` badges, inline figures, screenshot snippets pasted as HTML) into a monospace raw-source island, so HTML images never render there while Read mode renders standalone HTML image blocks through the shared HTML-parts pipeline. Users switching between Read and Visual Edit see images vanish into `<img src="…">` source boxes, which reads as "Visual Edit does not support HTML images".

## What Changes

- Inline `<img …>` tags inside prose blocks (paragraph, heading, list item, blockquote leaf, footnote text) render in Visual Edit as inline image atoms (same loader as Read mode: local paths, remote `http(s)`, `data:` URIs), with progressive source reveal: entering the tag's byte-exact source range reveals the authored `<img …>` markup as an editable run; surrounding prose stays rendered.
- Prose blocks whose only inline HTML is complete `<img …>` tags no longer fall back to a whole-block HTML source island. Any other inline HTML (`<br>`, `<em>…</em>`, bare `<a>…</a>` wrappers, comments, partial tags) keeps the existing whole-block source-island fallback.
- Standalone raw-HTML blocks containing images (e.g. `<p align="center"><img …></p>`) keep their current Visual Edit behavior — rendered read-only through the shared HTML-parts pipeline — now pinned by regression tests.
- GFM tables whose cells contain only `<img …>` tags no longer collapse the whole table into a source island; cells present the flattened alt/URL text exactly as Read mode does (no inline image inside table cells in this change).
- Visual Edit image-cache claims cover the new inline image runs so they preload, stay claimed while visible, and evict like other preview images.
- The Visual Edit support matrix (`docs/visual-editing-quality.md`) gains the inline-HTML-image strategy and stops classifying HTML as "always a complete source island".

### Non-goals

- Rendering general inline HTML (`<br>`, inline tags other than `img`) inside Visual Edit prose — remains the whole-block source island until a dedicated change.
- Rendering inline images inside GFM table cells (cell text stays flattened, matching Read mode).
- Changing Read mode / Split Preview behavior: inline `<img>` in prose still flattens to alt/URL text there; standalone HTML blocks are unchanged.
- New image presentation controls (resize/alignment) for inline HTML images — reveal-edit of the raw tag is the editing path.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: Visual Edit presentation requirements change — inline HTML images become rendered atoms with progressive reveal instead of whole-block source islands; the source-backed Visual Edit requirement and the support-classification contract gain the new classification.

## Impact

- `src/visual.rs` — `inline_runs` distinguishes image-only inline HTML from other inline HTML; new image run/reveal data on `VisualInlineRun`; projection reveal handling.
- `src/model.rs` — `VisualInlineRun` image payload, `VisualRevealKind` variant.
- `src/app/preview.rs` — inline image atom rendering in the mixed text/math element path; island gating unchanged for non-image HTML.
- `src/app/preview_image.rs` — cache-claim collection walks inline image runs.
- `src/parse.rs` — reuse/expose the exact `<img>` tag recognizer for the inline path.
- `docs/visual-editing-quality.md` — matrix row update.
- Invariants preserved: derived-state caching per document version (no recompute on caret movement), byte-exact source mapping, canonical-source-only mutations, no `gpui` dependency in `crates/*`.
