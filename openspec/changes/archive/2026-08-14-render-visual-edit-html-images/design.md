# Design: render-visual-edit-html-images

## Context

Read mode funnels raw HTML blocks through `html_preview_parts` (`src/parse.rs`), which lifts `<img>` tags into `HtmlPreviewPart::Image` rendered by `preview_image_view`. Since change `6220b33` Visual Edit maps `PreviewBlock::Html` to `VisualBlockKind::Html` and renders it through the same pipeline (unfocused; focused blocks keep the conservative source island per the "Complex constructs use conservative edit islands" scenario).

The remaining gap is *inline* HTML inside prose. `inline_runs` (`src/visual.rs`) treats any `Event::Html | Event::InlineHtml` as `contains_html`, and `visual_block_from_preview` then forces `VisualSourceIslandKind::Html`, which `visual_block_view`'s `always_source` gate renders as a whole-block raw-source box. So `Hello <img src="x.png"> world`, `- <img …>`, or `> <img …>` show raw source in Visual Edit while Read mode shows rendered prose (inline images flattened to alt/URL text) and rendered images for html-only paragraphs/blocks.

Editing model precedent: inline math (`VisualInlineRun.math`) renders as a baseline-aligned image atom while unfocused and reveals its byte-exact authored source as an editable run when the caret or a selection endpoint enters it (`build_visual_projection_with_marked_range` + `RevealCandidate`/`VisualRevealGroup` machinery). This change clones that pattern for `<img>` tags.

## Goals / Non-Goals

Goals:

- Inline `<img>` tags render as images inside Visual Edit prose blocks, using the existing preview image loader/cache.
- Byte-exact editing: the tag's authored source is the canonical editable range; caret/selection entry reveals it as source text; leaving restores the atom. No document reparse on caret movement (version/caches untouched).
- Whole-block source islands survive for any non-`img` inline HTML.

Non-goals (see proposal): general inline HTML rendering, images inside GFM table cells, Read-mode inline-image upgrades, image presentation controls for these atoms.

## Decisions

### D1: Represent inline images as a `VisualInlineRun` payload, not a new block kind

Add `html_image: Option<VisualHtmlImage { alt, url, title }>` to `VisualInlineRun`; `visible_text` is the byte-exact authored `<img …>` source and `source_range`/`content_range` both cover the tag. Alternatives:

- *New `VisualBlockKind`*: wrong granularity — inline images live inside prose flow with sibling runs; splitting blocks would break the "every source byte has exactly one visual owner" partitioning.
- *Extend `PreviewBlock::Paragraph` with image spans*: ripples into outline/stats/export/copy that consume `RichText`; Visual-Edit-only concern.

The math payload is the proven precedent: projection treats the run as one rendered piece; the view layer substitutes an atom for the segment.

### D2: Recognize exactly one complete `<img>` tag per inline-HTML event; otherwise keep the island

`inline_runs` on `Event::InlineHtml`: slice the event's source range and parse it with the existing tag parser (`ParsedHtmlTag`-based helper exposed from `src/parse.rs`). Emit an image run + `RevealCandidate` **only** when the slice is exactly one complete, non-closing `<img …>` tag with a non-empty `src` attribute (self-closing `/>` or plain `>` both accepted; `img` is a void tag). Track `contains_non_image_html` separately; the block keeps `VisualSourceIslandKind::Html` iff that flag is set, so `<img>` alongside `<br>` or `<em>` still falls back conservatively. Rationale: a narrow exact recognizer plus conservative fallback is the repo's stated parser-ownership policy; no second document parser is introduced.

### D3: Progressive reveal reuses the reveal-group machinery with a new kind

Add `VisualRevealKind::HtmlImage`; `reveal_candidate_is_exact` proves the slice starts with `<img` (case-insensitive) and ends with `>`. `build_visual_projection_with_marked_range` includes the kind in the `include_end` caret-endpoint rule (mirroring math, so the caret right after `>` keeps the tag editable). Focused reveal shows the raw tag in the inline-code source style, consistent with revealed math/link source.

### D4: Atom rendering rides the mixed text/math element path

`visual_text_with_math_element` already flex-waps projection segments into text fragments and math atoms; extend its trigger condition to image runs and add an image-atom branch. The atom wraps `preview_image_view` (ready/pending/error presentation for local/remote/data-URI URLs, `max_w_full`) with selection highlight and start/end hit targets that place the caret at the tag boundaries — mirroring `visual_math_atom`. `document_dir` is threaded from `visual_block_view` (it already holds it) into the element builder.

Alternative rejected: rendering the flattened alt text (Read-mode parity for mixed paragraphs). It would technically match Read mode byte-for-byte, but the user-facing ask is to *see the image*; the atom is a superset that keeps source mapping exact.

### D5: Image cache claims extend to inline runs

`collect_preview_image_urls` additionally walks visual blocks' `editable_runs` for `html_image` URLs. Claims therefore preload, hold, and evict inline images exactly like block-level images; no new cache, no GPUI image lifetime changes.

### D6: Tables take the no-island path automatically, cells stay flattened

A table whose only inline HTML is `<img>` no longer trips the island gate; the visual table renders with the flattened alt/URL cell text that `PreviewBlock::Table` already carries (identical to Read mode). The table editor's per-cell field still maps the cell's exact source, so focusing the cell edits the raw tag conservatively. No new table code.

## Data flow / caching

Derivation stays version-keyed: `visual_blocks()` derives from the cached per-version preview blocks; image runs and reveal groups are part of that derived output (`Arc`-shared). Caret movement, selection, and reveal change only the *projection* computed per frame from `(source, block, selection, cursor, marked_range)` — never the document version or caches. Image pixels flow through the existing `PreviewImageCache` claim/refcount lifecycle (`refresh_tab_image_claims` → `ensure_preview_images` → `preview_image_view`).

## Risks / Trade-offs

- [Large images inside prose lines can inflate row height] → The atom uses the existing preview image presentation (`max_w_full`); inline usage is opt-in by authoring. If it proves jarring, a later change can cap atom height.
- [Parsing drift between pulldown's InlineHtml event bytes and our tag recognizer] → Recognition is exact-slice (`ParsedHtmlTag` on the event range, single tag, `src` required); anything else degrades to the existing island, never to a wrong mutation.
- [Reveal/IME interactions on the revealed source run] → The revealed run flows through the same `VisualEditableText` projection as revealed math/link source; IME and caret affinity tests for those paths cover the mechanism.
- [Support matrix drift] → `docs/visual-editing-quality.md` row updated in the same change (contract requirement).

## Migration Plan

Pure additive presentation change; no persistence, format, or shortcut migration. Rollback = revert; source documents are never rewritten.

## Open Questions

(none)
