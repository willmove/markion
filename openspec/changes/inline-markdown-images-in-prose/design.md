## Context

`derive_preview_and_outline` currently pushes `PreviewBlock::Image` at `End(Image)` even when a paragraph, heading, or list item is open. The parent is flushed later with a source range that still covers `![…](…)`. Visual Edit then partitions those nested image leaves into stacked rows (`partition_prose_around_nested_images`). Read mode consumes the same extracted image blocks, so mixed source always becomes “image on one line, leftover prose on the next.”

HTML `<img>` already has the presentation we want: a `VisualInlineRun.html_image` atom inside mixed prose, with progressive reveal of the authored tag. Markdown `![…](…)` should use that path when it is not the only content of a paragraph.

## Goals / Non-Goals

**Goals:**

1. Mixed Markdown images stay inside the parent construct as inline atoms (same line as adjacent text) in Visual Edit and Read / Split Preview.
2. Authored `![…](…)` is not duplicated as a source island or as visible syntax under the preview.
3. Image-only paragraphs remain block-level `PreviewBlock::Image` (caption / width / alignment unchanged).
4. List items that contain Markdown images keep a single list row (no second bullet).

**Non-Goals:**

- Standalone image editor redesign.
- HTML table layout.
- Pixel-identical inline sizing vs Typora (reuse the HTML `<img>` atom rules).

## Decisions

### D1: Keep mixed images in the preview span stream

At `End(Image)`, attach an `InlineSpan` with an image payload to the open heading, list item, paragraph, quote, or table cell — the same routing as `push_preview_rich`. Do **not** push `PreviewBlock::Image` while a prose container is open.

When a paragraph flushes, if every remaining span is an image (no other prose), emit `PreviewBlock::Image` for each so image-only paragraphs keep the block-level editor. Headings and list items never convert; they keep the inline atom even when the payload is only an image.

**Why not keep extracting and only change Visual Edit:** Read / Split Preview would still stack. The user asked both surfaces to match Typora.

### D2: Visual Edit maps `Tag::Image` to the existing `html_image` atom

`inline_runs` already turns a complete raw-HTML `<img>` into a revealable image run. Handle Markdown `Start(Image)` / `End(Image)` the same way: one run covering the authored `![…](…)` bytes, `html_image` payload from dest/alt/title, `VisualRevealKind::HtmlImage` (or equivalent) so unfocused shows the atom and focused reveals source.

Once mixed images are no longer separate preview leaves, `partition_prose_around_nested_images` is a no-op for those shapes. Leave the partition as a safety net for any remaining extracted nested images.

### D3: Read / Preview mixed layout reuses the math+HTML-image flex wrap

`rich_text_with_math_element` currently special-cases only math. Treat `InlineSpan.image` like Visual Edit’s HTML image atom: `flex_none` / `max_w_full`, intrinsic size, not the block-level `preview_image_view` (`w_full`), which would force a full-width line break.

## Data flow / caching

```
MarkdownDocument.text
        │  per version, Arc-cached
        ▼
derive_preview_and_outline
  mixed Tag::Image → InlineSpan.image inside parent
  image-only paragraph → PreviewBlock::Image
        │
        ├─ Read / Split Preview: rich_text_with_math_element paints inline atoms
        ▼
build_visual_blocks
  partition (safety net)
  inline_runs: Tag::Image → html_image atom
        │  per version, Arc-cached
        ▼
visual_text_with_math_element
```

No new derived-state surface. Caret movement still only toggles reveal, not parse.

## Risks / Trade-offs

- **[Risk] `finish_rich_text` drops empty spans** → Mitigation: preserve image spans even when `text` is empty; do not trim them away as whitespace.
- **[Risk] List-item `End(Paragraph)` discards paragraph spans** → Mitigation: attach images to `list_item.spans` with the same priority as `push_preview_rich`.
- **[Trade-off] Mixed images lose per-image caption/width/alignment chrome** → Acceptable: those controls stay on standalone image-only blocks; mixed atoms match HTML `<img>` and Typora.
- **[Trade-off] `**prose**\n![img](url)` becomes one paragraph with a soft break plus an inline image** instead of two visual rows. Matches CommonMark/Typora.

## Migration Plan

None. Presentation-only. Rollback is reverting the parse + inline_runs + Read mixed-layout changes.

## Open Questions

None that block implementation.
