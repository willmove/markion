# Design: fix-visual-edit-inline-markdown-images

## Context

See proposal.md for motivation. Load-bearing facts from exploration:

- `derive_preview_and_outline` pushes `PreviewBlock::Image` at `End(Image)` even when the image is still inside an open paragraph or heading. The parent is flushed later with pulldown-cmark's full tag range, which still covers the image syntax. Empty image-only paragraphs are dropped; mixed parents are kept. After the existing stable sort, order is parent-then-image, but the ranges overlap.
- `inline_runs` has no `Tag::Image` arm. Re-parsing the unpartitioned parent therefore shows the alt as ordinary text (`Event::Text` inside the image) and ignores the destination.
- `build_visual_blocks` assumes disjoint leaf ranges. The overlap guard (`src/visual.rs`, `source_range.start < covered_until`) stamps `VisualSourceIslandKind::Unsupported`. `always_source` treats `Unsupported` like FrontMatter/Code, so the image is a gray source island even when unfocused.
- Nested-list partitioning already lives in this function; `fix-visual-edit-list-nested-code` truncated list items at nested *block* starts in the parse layer and explicitly left inline `![images]()` alone, because truncating a list item mid-sentence would emit a second bullet.

Read / Split Preview / export consume overlapping preview blocks independently and are in scope only as non-goals: they already show the user's adjacent-line case correctly.

## Goals / Non-Goals

**Goals:**

- Visual leaf stream for paragraphs and headings that swallow nested `PreviewBlock::Image` ranges is monotonically ordered and disjoint.
- Those images render through the existing `VisualBlockKind::Image` path; leftover prose after an image is a continuation visual row of the same kind (paragraph or heading), carrying quote context when the parent is a quote leaf.
- `inline_runs` on each slice no longer sees the nested image syntax, so alt text does not leak.
- Partition is part of per-version `visual_blocks()` derivation (`Arc`-shared). Caret movement does not re-parse or invalidate caches.

**Non-Goals:**

- Parse-layer split of `PreviewBlock` streams (would change Read/export paragraph semantics).
- Markdown image inline atoms (`html_image`-style). Same-line images become stacked rows.
- List-item parents (second-bullet problem). The overlap guard remains the fallback there.
- Focused-caret source-island behavior for standalone image-only blocks.

## Decisions

### D1: Partition in `build_visual_blocks`, not in the preview parser

Rewrite the expanded leaf list *after* quote expansion and the existing nested-list truncate, *before* the `covered_until` / gap / overlap-guard loop.

For each `Paragraph` or `Heading` leaf `P` whose computed source range strictly contains one or more following `Image` leaves `I1…In`:

```
P[start .. I1.start]     → same kind as P (skip if empty)
I1
P[I1.end .. I2.start]    → continuation, same kind / quote_group as P (skip if empty)
I2
…
P[In.end .. P.end]       → trailing continuation (skip if empty)
```

Continuation rows still point at the original `PreviewBlock`; only the `source_range` passed into `visual_block_from_preview` changes. `inline_runs` re-parses that slice, so markers, links, and quote prefixes on those bytes keep working.

**Why visual layer:** the overlap is a Visual Edit ownership problem. The preview stream's “parent plus extracted image” shape is what Read mode already renders. Splitting `PreviewBlock::Paragraph` in the parser would change export/`<p>` grouping and is out of scope.

**Why not inline atoms:** Visual Edit's Markdown image UI is already block-level (preview + caption). Approach B would duplicate the HTML `<img>` machinery and drop caption/alignment for mixed paragraphs. Stacked rows match the existing `VisualBlockKind::Image` contract.

**Alternatives considered:** (a) parse-layer paragraph split — rejected, changes Read/export; (b) skip emitting nested `Image` visual rows and hope the parent renders them — rejected, parent `inline_runs` has no image arm; (c) only truncate `P.end = I1.start` and leave trailing text to `gap_block` — rejected, mid-paragraph `world` would become an `Unsupported` island.

### D2: List items stay unpartitioned

A continuation `ListItem` would redraw a bullet; a continuation `Paragraph` would drop list indent. Both are worse than today's overlap quirk for a follow-up. Quote *paragraph children* are ordinary `Paragraph` leaves with `quote_group` set — they *are* partitioned, and every fragment copies `quote_group` so `quote_context_for_row` still attaches markers.

### D3: Keep the overlap guard as a safety net

After D1, paragraph/heading + image should never hit `source_range.start < covered_until`. Leave the `Unsupported` stamp in place for remaining overlaps (list-item images, anything new). Do not special-case `VisualBlockKind::Image` in `always_source`.

### D4: Image cache claims stay on `VisualBlockKind::Image`

`collect_preview_image_urls` already walks visual image blocks. Partitioned images remain that kind, so preload/claim/evict is unchanged. No new cache key.

## Data flow / caching

```
MarkdownDocument.text (canonical)
        │  per version, Arc-cached
        ▼
derive_preview_and_outline     ← unchanged; parent still swallows image ranges
        │
        ▼
build_visual_blocks
  quote expand → nested-list truncate → NEW image partition → gaps / overlap guard
        │  per version, Arc-cached
        ▼
visual_block_view
  VisualBlockKind::Image → preview_image_view (existing)
```

Caret, selection, and reveal still run against the cached visual blocks. Partition does not run on the keystroke path beyond the existing version invalidation.

## Risks / Trade-offs

- [Same-line `hello ![x](url) world` becomes three stacked rows] → Accepted; block-level image UI is the point of approach A. Inline atoms are a later change if this feels wrong in practice.
- [Heading + image yields two heading-styled rows] → Rare; keep heading kind so ATX markers on the leading slice still reveal correctly.
- [Whitespace-only interstitial slices become tiny paragraph rows] → Emit them anyway so bytes have an owner; empty (`start >= end`) slices are skipped. Trailing newlines stay on the preceding prose slice, matching current paragraph ranges.
- [List-item + image remains a gray island] → Documented non-goal; overlap guard still fires there.
- [Continuation heading/paragraph identity vs. block menu / reorder] → Each fragment gets its own `VisualBlockId` from the existing allocator; duplicate/delete/reorder already operate on one visual row's source unit. Verify a continuation row's source unit is the slice, not the original full paragraph (otherwise delete would eat the image). If `block_can_transform_at` uses the preview block's full range, restrict transform/reorder to fragments the same way nested-list rows already isolate ownership — pin with a test.

## Migration Plan

Pure derivation/presentation change. No persistence, settings, or source rewrite. Rollback = revert.

## Open Questions

None blocking.
