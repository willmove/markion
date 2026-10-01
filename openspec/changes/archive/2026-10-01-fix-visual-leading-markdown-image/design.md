## Context

`build_visual_blocks` already partitions a paragraph/heading around nested `PreviewBlock::Image` leaves (`partition_prose_around_nested_images` in `src/visual.rs`). That pass assumes the prose parent appears **before** its nested images in the expanded leaf list, and then only looks *forward* at consecutive `Image` leaves whose ranges sit inside the parent.

The preview stream does not guarantee that order when the image is the first construct in the paragraph. `derive_preview_and_outline` pushes `PreviewBlock::Image` at `End(Image)` while the paragraph is still open, then flushes the parent at `End(Paragraph)` with pulldown-cmark’s full tag range (which still covers the image). After the stable sort by `source_range.start`, equal starts keep parse order: **Image, then Paragraph**.

Reported fixture:

```markdown
![image.png](https://example.com/a.png)和其他瀚博半导体商标均为……
```

Today Visual Edit draws the image row, then the overlap guard stamps the still-unpartitioned parent `Unsupported`, so `always_source` paints the complete `![…](…)` syntax under the preview. Tests for `text ![alt](url) more` never hit this because the parent starts *before* the image.

## Goals / Non-Goals

**Goals:**

1. A paragraph or heading that starts with a nested Markdown image still partitions into disjoint visual rows (image, then leftover prose).
2. The authored image syntax is owned only by the image row — it MUST NOT also appear as a source island or as visible copy under the preview.
3. Existing cases (prose-before-image, image-only, blank-line-separated, quoted leaves) stay partitioned the same way.
4. Partition remains part of per-version `visual_blocks()` derivation (`Arc`-shared). Caret movement does not re-parse.

**Non-Goals:**

- Parse-layer reorder of `PreviewBlock` (would change Read/export).
- Markdown image inline atoms (`html_image`-style). Stacked rows stay the presentation.
- List-item inline Markdown images.
- HTML table rendering (separate thread).

## Decisions

### D1: Keep the fix in the visual partition, not in the preview parser

Same layer as `fix-visual-edit-inline-markdown-images`. The overlapping “parent plus extracted image” shape is what Read mode already consumes. Reordering preview blocks so Paragraph always precedes Image at equal starts would be a silent Read/export change and is rejected.

### D2: Consume contained images regardless of whether they sit before or after the parent in the leaf list

The current “look forward from the parent at consecutive Image leaves” scan misses images already passed.

Replace it with a containment scan:

1. When the cursor is an `Image` leaf whose range is contained in a **later** `Paragraph`/`Heading` leaf that is a partition parent, **do not emit** that image yet — the parent will own it.
2. When the cursor is a partition parent, collect **every** not-yet-emitted `Image` leaf (before or after in the list) whose range is contained in the parent range, sort those images by source start, and emit the same slices as today: prose before each image (skip empty), the image row, leftover prose after the last image (skip empty).
3. Image-only paragraphs still have no parent (empty paragraph dropped). Step 1 finds no later parent, so the image emits as a standalone row — unchanged.
4. List items remain non-parents. An image inside a list item is not “contained in a later paragraph/heading” in the usual case and still hits the overlap-guard fallback.

This preserves the existing output shape; it only fixes discovery/order.

**Alternatives considered:** (a) special-case equal start offsets by swapping two adjacent leaves — rejected, brittle with multiple images or quote expansion; (b) parse-layer `sort_by_key` that prefers Paragraph over Image on a tie — rejected, changes preview order; (c) exempt `VisualBlockKind::Image` from the overlap `Unsupported` stamp — rejected, would still leave the parent owning the image bytes and leaking syntax via `always_source` or `inline_runs`.

### D3: Keep the overlap guard

After D2, leading-image paragraphs should never hit `source_range.start < covered_until`. Leave the `Unsupported` stamp for remaining overlaps (list-item images, anything new).

## Data flow / caching

```
MarkdownDocument.text
        │  per version, Arc-cached
        ▼
derive_preview_and_outline     ← unchanged; Image may still precede parent
        │
        ▼
build_visual_blocks
  quote expand → nested-list truncate
  → partition (NOW: contained images before or after parent)
  → gaps / overlap guard
        │  per version, Arc-cached
        ▼
visual_block_view
```

No new derived-state surface. No invalidation on caret/hover.

## Risks / Trade-offs

- **[Risk] Skipping a leading Image that is not actually contained in a later parent** → Mitigation: containment is `image.start >= parent.start && image.end <= parent.end` on already-validated visual source ranges; image-only has no parent and is not skipped.
- **[Risk] Quote-expanded leaves copy `quote_group` onto extracted images** → Mitigation: keep the existing copy of the parent’s `quote_group` onto the image row; add a quoted leading-image fixture.
- **[Trade-off]** Still stacked rows, not baseline-aligned inline images. Matches the established Markdown-image Visual Edit contract; closing that gap is a separate change.

## Migration Plan

None. Presentation-only Visual Edit derivation. Rollback is reverting the partition change.

## Open Questions

None that block implementation. Manual check: the reported 瀚博 “商标声明” line in Visual Edit should show the image and the Chinese trailing sentence, with no `![image.png](https://…)` island under the preview.
