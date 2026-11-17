## Context

See proposal.md for motivation. Slash completion already uses `anchored()` and `document_overlay_anchor`; emoji completion uses fixed right/top coordinates and tall text rows. Source editing exposes cached shaped-line geometry through `layout_point_for_offset`, while Visual Edit stores painted caret and block bounds.

## Goals / Non-Goals

**Goals:** Keep the two completion surfaces visually aligned and tied to the active editing surface, without duplicating theme/layout constants.

**Non-Goals:** No replacement parser, new external dependency, document model, or source mutation path.

## Decisions

- Share a 248px themed popup shell and 36×30px compact controls with 2px grid gaps. Emoji completion shows 36 candidates at a time as six rows of glyphs, raising the overall popup from about 138px to about 266px. Tooltips and a selected shortcode caption retain discoverability; filtered results can occupy fewer rows.
- Share the below-source anchor and 8px offset with slash completion; let GPUI's anchored container fit the popup into the window. Resolve the query's block using the existing shared Visual block cache. In source mode, resolve the caret from the already shaped lines and source bounds, avoiding stale Visual Edit geometry.
- Hover only updates the transient selected result after validating the captured query, matching slash behavior. Keyboard navigation and confirmation continue through existing actions and checked source edits.
- Data flow: canonical source/caret → existing query state → cached geometry → shared popup shell/grid → existing checked completion transaction. Popup rendering, filtering, hover, and dismissal do not invalidate Markdown/highlighting/text caches or increment source version; confirmation retains normal versioned invalidation and one-step undo.
- Follow-up diagnosis: pulldown-cmark accepts unspaced `:bug:` as a definition-list opener after a term, stripping the token's colon before inline emoji recognition. Inspect raw definition events and protect only recognized shortcode opener bytes with an equal-length non-syntax sentinel before reparsing. Restore textual payloads before coalescing, retaining original byte offsets, code/HTML payloads, smart punctuation, literal control characters, and genuine definition markers. Ordinary documents retain the single-parse path. All work stays in the existing per-version derivation cache.
- Reuse the Markdown member crate's existing emoji map through a catalog accessor and sort the resolved application catalog, overlaying legacy aliases to avoid changing existing glyph choices. Completion, Visual Edit, preview and export share that catalog. Keep 36 visible candidates; a moving window follows the selected index so every matching candidate is reachable. Up/Down navigate individual matches, the pointer wheel advances a window, and the caption shows the selected position and total. The anchored container continues to fit the taller popup to the viewport.

## Risks / Trade-offs

- [Glyph-only buttons conceal names] → shortcode tooltip plus active caption.
- [Long shortcodes overflow the compact width] → truncate the caption while keeping the full tooltip.
- [Geometry is available only after paint] → retain the existing block/input fallback and frame refresh, and test source/Visual Edit positioning with rendered windows.
- [Viewport fitting can move the popup above the caret near the bottom] → use the same anchored fitting behavior as slash completion and cover constrained windows.

## Migration Plan

No migration. Revert the scoped rendering and geometry changes to restore the prior popup behavior.
