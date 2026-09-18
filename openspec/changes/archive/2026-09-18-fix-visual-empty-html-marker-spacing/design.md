## Context

See proposal.md — Why. Key facts the design builds on:

- `<a id="english"></a>` is a paragraph of inline HTML (CommonMark type-7 HTML blocks require the open tag to be followed only by whitespace; the closing `</a>` disqualifies it). pulldown emits `Start(Paragraph)`, `InlineHtml` × 2, `End(Paragraph)`.
- The `InlineHtml` events are consumed by `HtmlInlineState::handle` (which tracks `<a …>` frames for html links), so the paragraph draft keeps zero spans; `html_only_paragraph_source` then returns false because both tags classify as inline style tags (`needs_block_renderer` stays false), and the empty paragraph is dropped. The bytes become an `Unsupported` gap island whose chrome (border, `py_1`, `px_2`, `mb_2`, gray background) plus the swallowed trailing blank line produce the reported whitespace.
- Genuine empty HTML blocks (`<div></div>`) reach `visual_html_editor`, which always wraps `visual_collapsible_source_block` (bordered box, `mb_3`) and a gray `p_2` payload editor, even though `html_preview_parts` flattens them to nothing.
- Read/Split Preview already collapse empty parts to a bare zero-height div; only Visual Edit stacks chrome.

## Goals / Non-Goals

**Goals:** a content-free HTML marker occupies one slim, muted, monospace, fully editable source line with no island chrome or block margins, whichever parse path produced it (inline-HTML paragraph or true HTML block); content-bearing HTML is untouched; no duplicated flattening work per render.

**Non-Goals:** hiding the marker text, touching authored blank lines, changing inline HTML mixed with prose, or the supported inline-HTML subset.

## Decisions

1. **Route content-free tag-run paragraphs to the HTML block renderer.** `html_only_paragraph_source` now also returns true when the run contains no non-whitespace text at any depth (previously only when a tag was outside the inline style set). `<a id="…"></a>`, `<b></b>`, and whitespace-only containers become `PreviewBlock::Html`; `<b>bold</b>` and `<a href=…>label</a>` keep the ordinary styled-paragraph path because they carry visible text, and `<br>` tags count as visible text so break-only runs keep the paragraph path that paints their empty lines. Alternative — always accepting tag-only runs — was rejected: it would move editable prose like `<b>bold</b>` out of the inline reveal pipeline.
2. **Branch in `visual_html_editor` on visible content, not on tag names.** A helper treats parts as content-free when no Image/Table part exists and every Text part's rich text contains neither a non-whitespace character nor a `<br>`-authored newline. This covers anchors and empty containers without a tag-name allowlist that would drift.
3. **Render the payload field directly as the compact row.** The same `visual_editor_field_element` used by the payload editor provides caret, hit testing, IME, and projection; the row div supplies muted color, code font, and source-island metrics. The field's source range excludes the block's trailing line separator so the row paints exactly one line; the separator byte itself still maps to the line end, and later offsets belong to the following whitespace row. The collapsible wrapper and its `</>` toggle are dropped for this case — there is no presentation to toggle and the source is already visible.
4. **Compute parts once per render.** `html_preview_block_view` gains an internal `html_preview_block_view_with_parts` so `visual_html_editor` flattens once for both the emptiness check and the presentation; the no-editor path keeps the convenience wrapper.

## Risks / Trade-offs

- [Prose paragraphs of pure style tags (`<b></b>`) change presentation] → intentional: they previously dropped into an unsupported island with chrome; the compact marker row is strictly closer to their invisible rendering, and adding text restores the styled paragraph on the next parse.
- [Read/Split Preview] → the newly emitted `Html` preview block flattens to no parts there too, so rendered output stays visually identical (previously the dropped paragraph rendered nothing).
- [Losing the expand toggle on content-free blocks] → intentional: the collapsed state already shows the complete source; nothing is hidden to expand.
- [Geometry regressions] → a GPUI regression measures the anchor-row-to-heading caret span against a one-character paragraph row and asserts near-equality, plus a typed-input round-trip on the compact row.

## Migration Plan

Presentation-only; no document migration. Revert is the classification change plus the single rendering branch and test updates.
