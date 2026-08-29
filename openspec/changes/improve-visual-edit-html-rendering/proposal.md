# Proposal: improve-visual-edit-html-rendering

## Why

Visual Edit's HTML path is a narrow flattener plus a conservative source island, not a layout engine. Common authored HTML — sized README logos, tables wrapped in a `<div>`, images inside lists or quotes, and clicking an HTML block — still looks broken or dumps the user into a whole-block source box. After those P0 holes, everyday document HTML (headings, lists, `<pre>`, alignment/color/underline, table-cell images and links, classed `<em>`/`<br>`) and three tracked WYSIWYG gaps (unsupported inline HTML, residual entities, angle-bracket autolinks) keep Visual Edit from matching the rendered result users already expect from Read mode.

## What Changes

Work in three verified waves. Each wave ships with parse, Visual Edit, and (where the shared HTML pipeline is involved) Read/Split Preview tests.

**P0 — looks wrong today**

- Honor `<img width>` / `<img height>` (including the repository README logo at 128×128) on standalone HTML blocks and Visual Edit inline HTML image atoms.
- Focusing a Visual Edit HTML block keeps the rendered HTML visible and reveals a collapsible source payload (math/diagram pattern) instead of replacing the whole block with a bordered source island.
- HTML tables whose `<table>` is nested in a wrapper (`<div>`, comments, leading text) still resolve as grids; surrounding non-table HTML is not dropped.
- Raw HTML images inside list items and blockquotes render as images through the shared HTML-parts pipeline, not flattened alt/URL text.

**P1 — common document HTML**

- HTML `<h1>`–`<h6>` use heading typography derived from the rendered body size.
- HTML `<ul>` / `<ol>` / `<li>` show list markers (bullets / numbers).
- HTML `<pre>` preserves authored whitespace instead of collapsing it.
- Honor `align="left"` / `align="right"` (and equivalent `text-align`), inline `color` / `font color`, and underline (`<u>`).
- HTML table cells render `<img>` and `<a href>` instead of empty or unlinked text.
- Supported inline-HTML tags that carry ignorable attributes such as `class` (for example `<em class="x">`, `<br class="clear">`) still render and progressively reveal; they no longer collapse the containing prose block.

**P2 — WYSIWYG coverage gaps 6, 8, and 13**

- Unsupported / unknown / stray inline HTML presents as progressive-reveal or inert source atoms in the mixed layout; focusing no longer promotes the whole paragraph to a source island.
- Entity references outside the current proven single-codepoint table still project when they can be reconstructed against the parser (widen the named table; multi-codepoint names get a proven multi-character span or stay conservative).
- Angle-bracket autolinks (`<https://…>`, `<user@example.com>`) render as progressive-reveal links instead of whole-paragraph islands.

The Visual Edit support matrix and WYSIWYG coverage roadmap in `docs/visual-editing-quality.md` move each closed construct out of the gap class. Stale `markdown-editing` scenarios that still call decoded entities a primary gap, or that claim `<em>`/`<br>` keep a whole-block island, are brought in line with the implemented contract.

### Non-goals

- A general-purpose browser / CSS layout engine, scripts, event handlers, `iframe` / `video` / `embed`, inline SVG documents, `float`, `srcset` / `<picture>`, or nested HTML tables as true nested grids.
- Visual Edit cell editing for HTML tables (cells stay read-only rendered; source payload edits the authored HTML).
- Changing Markdown `![alt](url)` image presentation controls, GFM pipe-table editing, or HTML export semantics beyond consuming the richer preview parts.
- Closing unrelated roadmap gaps (front matter, indented code, malformed fences/tables, task-list checkbox click, definition lists, empty list items, math render-failure).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Visual Edit HTML block presentation and editing affordance; shared HTML-parts rendering (image size, alignment, structure, table cells, nested list/quote HTML); inline HTML subset (attributed supported tags); unsupported inline HTML, residual entities, and angle-bracket autolinks move from coverage gaps to rendered or progressive-reveal classes; support-classification matrix and roadmap update.
- `document-typography`: HTML headings, lists, and `<pre>` in the shared preview/Visual Edit HTML pipeline derive metrics from the resolved rendered body size with the same proportions as Markdown headings, lists, and code.

## Impact

- `src/parse.rs` — `HtmlPreviewPart` richness (image size, alignment, heading/list/pre/color/underline); table detection anywhere in a block; table-cell images and links; list/quote HTML classification in `src/lib.rs`.
- `src/visual.rs` / `src/model.rs` — HTML block editor + collapsible source; inline HTML attribute relaxation; unsupported-tag atoms; entity table / multi-codepoint spans; autolink reveal.
- `src/app/preview.rs` — HTML block view (keep render on focus), image sizing, heading/list/pre/alignment/color/underline painting, table-cell media.
- `docs/visual-editing-quality.md` — matrix and roadmap.
- Invariants preserved: per-document-version derived Markdown caches (no reparse on caret/hover), byte-exact source mutations only, `crates/*` remain GPUI-free.
