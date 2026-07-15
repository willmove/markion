## Context

`MarkdownDocument::derive_preview_and_outline` performs one offset-aware `pulldown-cmark` pass and stores the resulting preview blocks and outline in per-document-version caches. Raw HTML that is emitted as a standalone `Event::Html` already has a `PreviewBlock::Html` path, but README-style container markup can be emitted as inline HTML inside a parser paragraph. The current event handling then flattens that markup to plain text, so the image-bearing HTML never reaches the HTML preview renderer.

Separately, `HtmlPreviewBuilder` normalizes whitespace independently for each text node. Trailing whitespace before an inline tag is discarded at the end of one node, so `<strong>English</strong> · <a ...>简体中文</a>` becomes `English ·简体中文`.

The data flow remains:

`Markdown source + document version` → `single pulldown offset pass` → `PreviewBlock::Html` → `Arc<Vec<PreviewBlock>>` cache → `html_preview_parts` → GPUI preview.

## Goals / Non-Goals

**Goals:**

- Preserve source-faithful, standalone README-style raw HTML as an HTML preview block in document order.
- Keep mixed Markdown paragraphs with genuinely inline HTML on the existing rich-text path.
- Normalize visible HTML whitespace across adjacent text nodes without losing separators or incorrectly changing styling/link boundaries.
- Preserve the single-pass derivation and per-version cache behavior.

**Non-Goals:**

- Adding a browser engine or supporting arbitrary CSS, scripts, or the full HTML layout model.
- Changing HTML export output or the public preview-block model.
- Reparsing derived state during rendering or on non-text UI interactions.

## Decisions

### Classify HTML-only parser paragraphs as raw HTML blocks

Track whether a parser paragraph contains only inline HTML events plus ignorable whitespace. At paragraph completion, use its offset range to retain the original source and emit one `PreviewBlock::Html`; if ordinary Markdown text or another construct appears, keep the existing paragraph/rich-text behavior. This handles `pulldown-cmark`'s event classification without relying only on whether the event variant is `Html` versus `InlineHtml`, and keeps the original attributes, tag order, and whitespace available to the renderer.

Alternative considered: treat every `InlineHtml` event as a separate HTML block. This would split one logical container into multiple preview blocks and would break normal inline HTML embedded in prose.

### Carry pending collapsible whitespace across HTML tokenizer boundaries

Move pending whitespace from a local `push_text` variable into `HtmlPreviewBuilder` state. Resolve it when the next visible token or inline tag boundary is handled, using the style/link context in which the whitespace occurred; block boundaries and explicit line breaks continue to use the existing newline normalization. This preserves one visible separator while preventing runs of formatting whitespace from becoming multiple spaces.

Alternative considered: concatenate all text nodes before styling. That would lose the existing per-span bold, code, strikethrough, and link metadata.

### Keep regression tests at both transformation boundaries

Retain focused tests for Markdown events → preview blocks and raw HTML → preview parts, then run the full workspace suite. These boundaries isolate parser classification failures from HTML text-normalization failures.

## Risks / Trade-offs

- [An HTML-only paragraph is misclassified when it contains meaningful plain text outside tags] → Mark the paragraph as mixed as soon as a non-whitespace Markdown text event occurs and cover mixed inline HTML with a regression test.
- [Pending whitespace acquires the style or link of the following tag] → Resolve pending space before applying the next inline tag's style/link transition and assert span metadata as well as flattened text.
- [Raw HTML handling causes an extra parse or invalidates caches] → Perform classification inside the existing offset pass and keep the cache installation/version checks unchanged.
