## 1. P0 — Image dimensions

- [x] 1.1 Parse `width`/`height` on `<img>` into `HtmlPreviewPart::Image` and `VisualHtmlImage` (px and percent)
- [x] 1.2 Apply those sizes in `preview_image_view` and `visual_html_image_atom` with `max_w_full`
- [x] 1.3 Tests: README logo 128×128, width-only aspect, inline Visual Edit atom, pending placeholder unchanged

## 2. P0 — HTML block focus keeps render

- [x] 2.1 Add `VisualBlockEditor::Html` / `VisualEditorFieldKind::HtmlSource` over the full HTML source range
- [x] 2.2 Exclude `VisualBlockKind::Html` from `focused_conservative`; wrap `html_preview_block_view` in `visual_collapsible_source_block`
- [x] 2.3 Tests: focused HTML table/image still `VisualBlockKind::Html` with no source island; payload edit mutates source; hover/focus does not bump version

## 3. P0 — Wrapped HTML tables

- [x] 3.1 Scan HTML blocks for `<table` anywhere; parse grid from that offset; emit prefix/table/suffix parts
- [x] 3.2 Tests: `<div><table>`, comment-prefixed table, caption after table, nested table flattens safely

## 4. P0 — List and quote nested HTML images

- [x] 4.1 Promote HTML-only regions inside quotes and list items to disjoint `PreviewBlock::Html`
- [x] 4.2 Stop flattening image-bearing HTML inside lists/quotes through `html_preview_plain_text`
- [x] 4.3 Tests: nested `<p align="center"><img>` in a list item and a blockquote render as Html blocks with disjoint ranges; mixed `- hello <img>` still uses the inline atom path

## 5. P1 — HTML structure, alignment, color, underline

- [x] 5.1 Extend HTML text parts with heading level, list markers, `pre` whitespace, left/right/center align, underline, allowlisted color
- [x] 5.2 Render those parts in `html_preview_block_view` using document typography metrics
- [x] 5.3 Tests: h1 size, ul/ol markers, pre spaces/newlines, `align="right"` + `<u>`, hex/`rgb()` color; ignored `url()` colors

## 6. P1 — Table-cell images and links

- [x] 6.1 Parse `<img>` and `<a href>` inside HTML table cells; render images and link spans
- [x] 6.2 Tests: image-only cell, linked cell text; GFM pipe-table HTML images stay flattened

## 7. P1 — Attributed supported inline HTML

- [x] 7.1 Accept ignorable `class`/`id`/`clear` on supported inline tags without spoiling the block
- [x] 7.2 Tests: `<em class="x">`, `<br class="clear">`; non-ignorable attributes remain conservative runs

## 8. P2 — Unsupported inline HTML atoms

- [x] 8.1 Stop promoting `contains_non_image_html` to a whole-block `Html` source island; keep inert conservative runs
- [x] 8.2 Tests: `Hello <span>x</span> world` stays mixed layout focused and unfocused; unpaired tags do not island the paragraph

## 9. P2 — Residual entities and autolinks

- [x] 9.1 Widen `NAMED_ENTITY_DECODES`; prove multi-codepoint named entities as one reveal span
- [x] 9.2 Accept pulldown angle-bracket autolinks in the link reveal validator
- [x] 9.3 Tests: previously unlisted named entity; multi-codepoint name; `<https://example.com>` and email autolink; unknown entity still conservative

## 10. Matrix, specs hygiene, quality gate

- [x] 10.1 Update `docs/visual-editing-quality.md` matrix and roadmap (remove gaps 6, 8, 13; HTML focus/editor; nested HTML)
- [x] 10.2 `openspec validate improve-visual-edit-html-rendering` and `pwsh ./scripts/check-quality.ps1`
