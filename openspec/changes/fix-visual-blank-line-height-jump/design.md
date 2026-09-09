## Context

See `proposal.md` for motivation. Visual Edit currently gives every `Paragraph` block a `paragraph_spacing` bottom margin while `Whitespace` rows carry only line height. Typing into a whitespace row can either create a standalone paragraph (between headings) or merge adjacent paragraph blocks into one CommonMark paragraph (between prose). The number of bottom margins therefore changes even though the same authored visual lines remain.

The source-backed data flow remains:

`MarkdownDocument.text` → versioned `visual_blocks_shared()` cache → virtual-list row render → GPUI measurement / hit testing.

Only the last stage needs different geometry. Pointer placement without input remains selection-only; typed input still mutates canonical source through the existing edit path and creates the new document version normally.

## Goals / Non-Goals

**Goals:**

- Preserve the total occupied height of the affected paragraph/whitespace flow when a blank line becomes text.
- Keep whitespace content, caret height, and click-to-source stride at body line height.
- Apply paragraph spacing exactly once per contiguous, same-context paragraph/whitespace body flow.
- Lock the user-visible no-jump behavior with rendered GPUI bounds.

**Non-Goals:**

- Changing paragraph parsing, source ranges, or canonical newline bytes.
- Changing Read/Split Preview layout or non-paragraph block margins.
- Adding cached typography state to `VisualBlock`.

## Decisions

### Assign paragraph spacing to the end of a Visual Edit body-flow group

Define a Visual Edit body-flow group as consecutive `Paragraph` and `Whitespace` blocks whose structural context matches (top-level with top-level, or the same quote group/depth). Only the last block in that group receives `paragraph_spacing`; earlier paragraph and whitespace rows receive zero bottom margin. Each row keeps its own existing content height. This makes margin count independent of whether typing causes parser blocks to split or merge.

For `Paragraph → Whitespace → Paragraph`, the final paragraph owns the one margin. Typing into the whitespace produces one multi-line paragraph that still owns one margin. For `Heading → Whitespace → Heading`, the whitespace is a one-row body-flow group and owns the margin; typing produces a standalone paragraph that owns the same margin. Leading/trailing whitespace follows the same rule.

The whitespace caret stays `paragraph_line_height` tall, later-line offsets remain multiples of that line height, and pointer Y mapping continues to divide by the same line-height stride. The grouping helper reads the already-cached visual block slice during rendering and does not create derived state or change document versions.

Alternative considered: remove paragraph bottom spacing from paragraphs. Rejected because that changes established paragraph typography globally and affects Read/Preview parity rather than correcting the blank-line replacement state.

Alternative considered: add a paragraph margin to every whitespace row while leaving every paragraph margin intact. Rejected by the prose regression: typing merges neighboring paragraphs and removes two margins, so later content still moves upward.

Alternative considered: fold paragraph spacing into each whitespace line's internal stride. Rejected because the rendered paragraph's margin is external to its line box; doing so would change caret and click geometry and multiply spacing across multi-line whitespace rows.

Alternative considered: preserve line-height-only whitespace rows and compensate scroll position after typing. Rejected because it hides a geometry mismatch and would still produce remeasurement instability outside the active viewport.

### Prove stability at the rendered-list seam

GPUI tests cover both parser outcomes: a heading/blank/heading fixture where typing creates a standalone paragraph, and a paragraph/blank/paragraph fixture where typing merges three visual lines into one CommonMark paragraph block. Both capture a later stable block and assert its top coordinate is unchanged. Existing helper tests continue to cover line-height scaling, multi-line caret stride, and pathological bounds; a grouping helper test covers margin ownership and structural boundaries.

This seam exercises the parser ranges, whitespace hit testing, canonical input mutation, virtual-list splice, and final layout; a helper-only assertion would not catch the reported jump.

## Diagnosis Evidence

`cargo test visual_edit_typing_into_blank_line_preserves_row_height -- --nocapture` failed initially: the blank and replacement paragraph row bounds had equal heights, but the following heading moved from `118.5px` to `130.5px`. The exact `12px` delta showed that margin ownership—not line-box or caret chrome—caused the jump. Adding a margin only to whitespace fixed that fixture, but `visual_edit_typing_between_paragraphs_preserves_following_content_position` then showed the prose case moving from `190px` to `166px`: adjacent paragraph blocks merged and removed two existing margins. Together these probes require stable one-per-body-flow margin ownership.

## Risks / Trade-offs

- [Risk] Grouping across a quote or other structural boundary could suppress required spacing. → Mitigation: group only paragraph/whitespace neighbors with the same top-level or quote-group/depth context.
- [Risk] Applying spacing inside the whitespace content box could select the wrong newline in multi-line whitespace. → Mitigation: keep height/caret/click strides unchanged and vary only each row's external bottom margin.
- [Risk] Virtual-list estimates may retain stale whitespace measurements after typography changes. → Mitigation: reuse the existing typography remeasurement/reset path and verify the rendered bounds after input.
