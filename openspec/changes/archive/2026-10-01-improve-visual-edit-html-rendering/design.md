## Context

Visual Edit and Read/Split Preview share `html_preview_parts` (`src/parse.rs`) for standalone `PreviewBlock::Html`. That flattener emits only `Text` / `Image` / `Table`, recognizes centering but not size, routes tables only when the block *starts* with `<table`, and drops list/quote HTML to `html_preview_plain_text`. Visual Edit maps `PreviewBlock::Html` to `VisualBlockKind::Html` with empty runs; `focused_conservative` then replaces the render with a whole-block source island. Inline HTML in `src/visual.rs::inline_runs` proves a narrow unattributed subset; everything else sets `contains_non_image_html` and, on focus, a paragraph island.

Derived preview/visual blocks stay cached per document version. Presentation-only focus/hover must not bump version or rebuild those caches.

Data flow (unchanged cache boundary):

```
Markdown source + version
  → pulldown offset pass (src/lib.rs)
  → PreviewBlock::Html | prose blocks   [cached Arc]
  → html_preview_parts / inline_runs    [pure, per render of cached HTML string]
  → GPUI (html_preview_block_view / visual_block_view)
```

## Goals / Non-Goals

**Goals:**

- P0: image width/height; keep HTML render on focus with a collapsible source payload; tables found inside wrappers; HTML images in lists and quotes.
- P1: HTML headings, lists, `<pre>`, left/right align, color, underline; table-cell images and links; classed supported inline tags.
- P2: close roadmap gaps 6 (unsupported inline HTML), 8 (angle-bracket autolinks), 13 (residual entities).
- Tests at parse, document/visual projection, and GPUI view layers; matrix/roadmap updated.

**Non-Goals:**

- Browser CSS, scripts, iframe/video, inline SVG documents, float, srcset, nested HTML tables as nested grids.
- HTML-table *cell* editing (source payload edits the authored block).
- Unrelated roadmap gaps (front matter, indented code, malformed fences/tables, task checkboxes, definition lists, empty items, math failure).

## Decisions

### D1 — Image `width` / `height` as optional display hints on the image part

**Choice:** Extend `HtmlPreviewPart::Image` and `VisualHtmlImage` with `width_px: Option<f32>` and `height_px: Option<f32>`. Parse `width`/`height` as a CSS pixel length (`128`, `128px`) or a percentage of the loaded image’s display width. Apply in `preview_image_view` and `visual_html_image_atom` with `max_w_full()`. If only one dimension is set, keep aspect from the decoded entry.

**Why:** The README logo already authors `width="128" height="128"`; the parser currently ignores both. Putting size on the part keeps export/plain-text consumers ignorant of layout.

**Rejected:** Markdown `ImagePresentation` percent controls for HTML images — those rewrite Markdown syntax; HTML size lives on the tag. Reveal-edit remains the mutation path.

### D2 — HTML blocks keep the rendered view; source is a collapsible payload editor

**Choice:** Exclude `VisualBlockKind::Html` from `focused_conservative`. Attach `VisualBlockEditor::Html { payload }` covering the full block `source_range` (`VisualEditorFieldKind::HtmlSource`). The view wraps `html_preview_block_view` in the existing `visual_collapsible_source_block` used by diagrams/math: render always visible; `</>` / focus expands an exact source payload. Hover/focus does not change document version.

**Why:** Replacing a table or logo with a gray island is the worst P0 UX. The diagram pattern already solves “render + exact source edit” without a second document model.

**Rejected:** Always-on raw island; in-place rich-text mutation of HTML.

### D3 — Find `<table>` anywhere in an HTML block; emit interleaved parts

**Choice:** Replace the `starts_with("<table")` gate with a scan: for each `<table` tag, try `parse_html_table_grid` on the slice starting there; on success emit flattened parts for the prefix, one `HtmlPreviewPart::Table`, then continue after the parsed table’s end offset. Nested tables still fail the existing parser (`failed = true`) and flatten.

**Why:** `<div align="center"><table>…` and comment-prefixed tables are common. Returning *only* the table dropped captions; interleaved parts fix that.

**Rejected:** A full HTML5 tree. Too large; the flattener stays the default.

### D4 — List- and quote-nested HTML uses the same Html block stream as top-level

**Choice:** Promote HTML-only paragraphs inside blockquotes the same way top-level `html_only_paragraph_source` does (quote children become `PreviewBlock::Html`). For list items, emit nested `PreviewBlock::Html` in document order with disjoint source ranges, mirroring nested fenced code (`list item range` ends before the HTML block). Mixed prose+`<img>` inside a list/quote leaf stays on the inline-image atom path (source re-parse), not flattened via `html_preview_plain_text`.

**Why:** Today `Event::Html` inside list/quote is flattened, so images never reach `html_preview_parts`, and wrapping `<p align=center><img>` cannot prove a single img tag.

**Rejected:** Special-casing only `<img>` in `html_preview_plain_text` — tables and centered wrappers would still break.

### D5 — Richer `HtmlPreviewPart::Text` metadata, still not a DOM

**Choice:** Extend text parts (and table-cell `RichText` / spans) with:

- `heading_level: Option<u8>` for `h1`–`h6`
- `list_marker: Option<HtmlListMarker>` (`Disc` or `Decimal { n }`) on `li` close
- `pre: bool` — skip collapsible whitespace inside `<pre>`
- `align: HtmlAlign { Start, Center, End }` from `align` / `text-align`
- span extras: `underline`, optional RGB `color` from `style="color:…"` / `<font color>`

Renderer maps these to existing typography metrics (heading sizes, code font for pre) and GPUI styles.

**Why:** Enough for README-class HTML without a box model.

### D6 — Table cells reuse image/link extraction

**Choice:** In `HtmlTableParser`, handle `img` (push a cell-level image note or embed alt+url as a structured cell image) and `a href` (set span.link). Cell images render with the same `preview_image_view` sizing as D1. GFM *pipe* table cells stay flattened unless we later lift them; this change targets HTML `<td>`/`<th>` only. GFM cells that contain HTML `<img>` keep Read-mode flattened text (existing Visual Edit table invariant) unless they already go through HTML parts — do not collapse the GFM table.

**Why:** Design for rowspan already promised links; images were an accidental hole (`img` fell through `_ => {}`).

### D7 — Supported inline tags may carry ignorable attributes

**Choice:** `parse_inline_html_style_tag` accepts `class`, `id`, and `clear` (common on `<br>`) without mapping them to style. Other attributes still reject. Reveal group covers the full authored tags including attributes.

**Why:** GitHub-flavored paste (`<br class="clear">`, `<em class="…">`) currently spoils the block.

### D8 — Unsupported inline HTML is an inert/reveal atom, not a paragraph island

**Choice:** Stop promoting `contains_non_image_html` to `VisualSourceIslandKind::Html` for the whole block. Unknown/stray tags stay `conservative_fallback` runs (verbatim source visible, progressive reveal of that tag range). Unpaired/crossing tags still demote inner runs but do not set `always_source` / focused whole-block island. Malformed slices that fail reconstruction stay conservative runs, not FrontMatter-style islands.

**Why:** Roadmap gap 6’s target class is progressive-reveal / inert atoms.

### D9 — Residual entities: widen the named table; multi-codepoint as one reveal span

**Choice:** Extend `NAMED_ENTITY_DECODES` toward HTML5 single-codepoint names used in real docs. For names that decode to multiple chars, add `DecodedSpan` with `visible_text` equal to the parser’s string and `content_range` the full `&…;` token (same proof as single-codepoint). Invalid/unknown still conservative.

**Why:** Gap 13 is documented as data-only plus multi-char spans.

### D10 — Angle-bracket autolinks share link reveal

**Choice:** Teach the link reveal validator to accept `LinkType::Autolink` / email autolink sources shaped `<url>` / `<email>`. Render the visible destination as a link; reveal the full `<…>` group on caret entry.

**Why:** Gap 8’s seam is `src/visual.rs` invalidation that requires `[`-shaped sources.

## Risks / Trade-offs

- **[HTML payload editor mutates a large block]** → One field covering the whole HTML range; same dirty/undo path as code payloads. No inferred DOM edits.
- **[Percent width without a loaded image]** → Apply percent after decode; pending state keeps the existing chip.
- **[List nested HTML overlapping ranges]** → Reuse the nested-code partition tests; fail closed to flattened text only if range proof fails.
- **[Color parsing injection]** → Allowlist hex / `rgb()` numeric only; ignore `url(` and expressions.
- **[Autolink vs HTML tag ambiguity]** → Only accept when pulldown already emitted a link event for that range.
- **[Cache]** → All new fields are derived from the cached HTML string or source slice at render/projection time; caret/hover still must not reparse the document.

## Migration Plan

Pure presentation + projection. No persistence format change. Rollback is revert. Update `docs/visual-editing-quality.md` in the same change as the code.

## Open Questions

None blocking: nested HTML tables stay flattened (non-goal); GFM pipe-cell HTML images stay flattened (parity with Read).
