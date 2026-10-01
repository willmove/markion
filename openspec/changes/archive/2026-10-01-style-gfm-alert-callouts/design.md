## Context

- Parsing: `PreviewBlock::BlockQuote { alert: Option<AlertKind>, .. }` already carries the kind (pulldown-cmark `ENABLE_GFM`, strict GFM markers only).
- Split Preview / Read (`preview.rs`, `PreviewBlock::BlockQuote` arm, around line 7683) ignores `alert` and draws a gray `border_l_1` quote.
- Visual Edit splits a quote into rows. Each row carries a `VisualQuoteContext { depth, marker_ranges, leaf_source_range, group_source_range, edge }`. The marker line is its own `VisualBlockKind::CalloutTitle { kind }` row, drawn as a bold accent label (`callout_label`, `callout_accent_color`). Every row is wrapped by `visual_quote_row` (around line 4763) with the same gray rule.
- Icons: Lucide SVGs embedded through `icon_set!` in `src/ui/icon.rs`.

## Goals / Non-Goals

**Goals:** one shared callout style (accent, tint, icon, label) used by both surfaces; fold toggle in preview only; no new parsing.

**Non-Goals:** Obsidian syntax extensions; Visual Edit folding (see proposal).

## Decisions

### 1. Shared style helpers
`callout_accent_color(kind)` stays the single source of accent colors (GitHub's palette, already validated for light and dark). New helpers:
- `callout_icon(kind) -> Icon`: Note → `info`, Tip → `lightbulb`, Important → `message-square-warning`, Warning → `triangle-alert`, Caution → `octagon-alert`. These are Lucide counterparts of GitHub's Octicons.
- `callout_background(kind) -> Background`: a left-to-right `linear_gradient` from the accent at about 10% alpha to about 3% alpha. Alpha tints keep contrast acceptable on both theme modes without per-theme colors.
- `callout_title_row(kind, …)`: icon plus bold label in the accent color.

The preview card and the Visual Edit quote rows both use these helpers, so the two surfaces cannot drift apart.

### 2. Visual Edit: the alert kind on the quote context
Add `alert: Option<AlertKind>` to `VisualQuoteContext`, set where `visual.rs` already builds quote contexts from the `BlockQuote` preview block. The value is known there (`alert: *alert`), so this adds no parsing and no extra derived pass. `visual_quote_row` uses the accent border (2 px) and the tint when `alert` is set, and the existing gray 1 px rule otherwise. Nested quotes take the innermost quote's alert, which is the context's own.

*Alternative:* look up the alert kind at render time by scanning the blocks for the group's `CalloutTitle`. Rejected: that is per-frame work and couples row rendering to sibling rows.

### 3. Preview fold state
Per-tab `folded_preview_alerts: HashSet<usize>`, keyed by the alert block's source start offset. A source offset survives re-render and scroll, and avoids a stable-id dependency the preview list does not have. It lives on `EditorTab` next to the other per-tab view state and is never persisted. After an edit in Split mode shifts offsets, a stale key can at worst leave a different alert at that exact offset folded. The fold toggles are user-visible, so this is self-correcting, and in Read mode the document does not change. The title row is a clickable element with a chevron (`chevron-down` or `chevron-right`) that calls `cx.notify()` only.

*Alternative:* fold by `[!NOTE]-` syntax. Rejected for this change: pulldown-cmark does not recognize it (see proposal non-goals).

### 4. Caches and data flow
Rendering reads `alert` from already-cached `PreviewBlock`s and `VisualBlock`s. The fold toggle mutates only the tab's UI set, followed by a notify. No document mutation, no version bump, and no invalidation of the per-version preview, visual, or outline caches. The preview list's measured row heights change when a callout folds, which is the normal effect of `notify` re-measuring a changed row. The Visual Edit `VisualQuoteContext` gains one `Copy` field, computed in the same pass that already builds it.

## Risks / Trade-offs

- [The tint reduces contrast for colored text inside callouts on dark themes] → 3–10% alpha keeps the background within a few percent of the theme surface. Body text keeps the existing quote text color.
- [Fold keys are offsets] → Accepted as described above. A future stable block id could replace them without spec changes.
