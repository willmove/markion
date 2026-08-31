## Context

Visual Edit stores the caret in canonical source offsets and paints it on a virtualized `ListState`. `stabilize-visual-edit-caret-viewport` already geometry-gates pointer placement: a click on a painted mid-document row no longer pins that row to the top. Typing still jumps, because the edit path and the identity path conspire to make the current row look like an unmeasured tail.

```
keystroke / IME update
  -> replace_source_range (version++)
  -> visual_blocks rebuilt; edited block gets VisualBlockId::fresh()
       (reconcile_visual_block_ids only copies ids when the source slice is unchanged)
  -> visual_block_splice keys on id → splice that one row
  -> GPUI ListState.splice replaces it with ListItem::Unmeasured (height 0)
       and, if the splice covers scroll top, resets offset_in_item to 0
  -> after_document_changed
       visual_cursor_reveal_pending = true
       visual_caret_bounds = None
  -> same render: apply_visual_caret_reveal
       item_bounds = None
       caret = None
       unmeasured_below = item_ix > scroll_top.item_ix   // true for every later visible row
       -> PinItem  // current paragraph jumps to the viewport top
```

`map_unchanged_range` returns `None` when the edit sits inside a block, so the edited row is invisible to today’s identity reuse. GPUI does re-measure already-visible items every frame; splicing them to `Unmeasured` is unnecessary for in-place text and is what creates the fake tail.

**Data flow (caching / versioning):**

```
in-place source edit
  -> version++ ; derived visual-block Arc rebuilt once (existing invariant)
  -> reconcile: 1:1 same-kind successor keeps VisualBlockId
  -> visual_block_splice is a no-op for that row (id + height_signature unchanged)
  -> last painted caret bounds stay on the tab
  -> geometry gate: caret still inside inset → logical_scroll_top unchanged
  -> visible-row layout remasures the Measured item in place

Enter / split / kind change
  -> new identities for affected successors
  -> splice inserts/replaces those rows (Unmeasured)
  -> gate uses last caret rect first; PinItem only if there is no geometry
     and the target sits below the previously measured window
```

Derived `Arc<VisualBlock>` caches, highlight memoization, and the cached text handle stay keyed on `MarkdownDocument.version()`. Identity reuse and scroll decisions MUST NOT reparse or drop those caches.

Related in-progress change: `fix-visual-edit-tail-fidelity` still splices identity-preserved whitespace rows when `height_signature` changes. That remasurement stays; this change only stops giving in-place prose/list/heading successors a new id.

## Goals / Non-Goals

**Goals:**

- Keep `VisualBlockId` across a proven 1:1 in-place successor of the same kind so ordinary typing does not splice the current list row.
- Reorder the caret geometry gate so a temporarily unmeasured row that was already in the visible window is not pinned to the top.
- Keep last-painted caret bounds across mutations and use them when `bounds_for_item` is `None`.
- Preserve last-line pixel-follow and true tail-row pin so `fix-visual-edit-tail-fidelity` still works.

**Non-Goals:**

- Compensating progressive-reveal glyph shift so the edited character stays under the pointer.
- Smoothing heading/list/code structure transitions (those still take new identities).
- Reusing identity across splits, merges, kind changes, or ambiguous reparses.
- Changing Source, Read, or Split Preview scroll; typewriter mode; Sync scroll; Enter insertion semantics.

## Decisions

### D1 — Reuse identity for 1:1 in-place same-kind successors

Today `reconcile_visual_block_ids` copies an old id only when `map_range_through_edits` succeeds (edit wholly outside the block) and the shifted clone equals the new block. Add a second, conservative matcher:

1. Keep the existing exact-slice pass for unchanged shifted blocks.
2. For remaining unmatched old blocks, map a *containing* range: if the pending edit is wholly inside `old.source_range`, expand that range by the edit delta; if the edit overlaps a boundary, skip.
3. If exactly one new block has `source_range == mapped_range` and `kind` equal to the old kind, and that new block is not already claimed, copy the old id.

Rejected: hashing visible text. Duplicate paragraphs would steal each other’s ids (`Repeated equal blocks remain occurrence-safe`).

Rejected: reusing id across kind changes (`Paragraph` → `Heading`). Those rows change chrome and height; a splice is the correct invalidation. The geometry gate (D2) still stops that splice from pinning a still-visible row.

Whitespace rows that keep their id while `height_signature` changes continue to splice via the existing `(id, height_signature)` key. That is the tail-fidelity remasurement contract, not this bug.

`source_mapped` stays GPUI-free. Tests live next to `reconcile_visual_block_ids`.

### D2 — Geometry gate prefers last caret rect; pin only below the measured window

`visual_caret_scroll_action` currently returns `PinItem` whenever `unmeasured_below` is true, *before* looking at caret bounds. After a splice that is always true for any row later than scroll top, even when the previous frame painted a caret in the middle of the viewport.

Reorder:

```
usable caret rect or item bounds (height > 0)?
  yes -> pixel delta or None (existing inset test)
  no, and item sits below the last measured window?
    yes -> PinItem
    no  -> RevealItem only when the viewport has height and the item is unknown
           otherwise None
```

“Below the last measured window” means `item_ix` is greater than the last index that `bounds_for_item` (or the list’s measured suffix) could resolve *before* this splice, not merely `item_ix > logical_scroll_top.item_ix`. A just-spliced index that was inside the previous visible range is not a tail row.

`after_document_changed` MUST NOT clear `visual_caret_bounds`. The next consume of `visual_cursor_reveal_pending` then has a one-frame-stale but in-viewport rectangle. `visual_caret_follow_frames` still runs after paint for last-line clip; it remains a no-op when the new caret stays inside the inset.

Rejected: dropping `visual_cursor_reveal_pending` on every mutation. Off-screen search/outline/keyboard still need the consume side; the consume side decides.

Rejected: special-casing “pointer vs typing” in `move_to`. The remaining failure is geometric (unmeasured + later index), same as the click bug.

### D3 — Reveal flags stay ephemeral; caches stay per-version

`visual_cursor_reveal_pending`, `visual_caret_follow_frames`, and `visual_caret_bounds` remain per-tab interaction state. Updating them MUST NOT increment `MarkdownDocument.version()` or drop derived caches. Identity reuse runs inside the existing `visual_blocks_shared` rebuild that already happens once per version.

## Risks / Trade-offs

- [Containing-range mapper reuses an id across a silent split] → Require exact `source_range == mapped_range` and equal `kind`; overlapping boundary edits skip. Add split/merge/kind-change tests that assert new ids.
- [Stable id on an in-place wrap leaves a stale off-screen height] → GPUI remasures visible items every frame; the caret row is kept in view. Whitespace already remasures via `height_signature`. Accept stale height only for off-screen non-whitespace rows the user is not typing in.
- [Keeping stale caret bounds pins the wrong place after a large scroll-unrelated mutation] → Bounds are window coordinates from last paint of this tab; a mutation that moves the caret off-screen still fails the inset test and pixel-follows or reveals. Whole-document replace already rebuilds the list.
- [Kind-change splice still resets `offset_in_item` when the edited row is scroll top] → Residual GPUI `splice` behavior; rare (typing `# ` at the start of a long wrapped paragraph that is the scroll top). D2 prevents pin-to-top; do not fight GPUI’s scroll-top reset in this change.
- [Pixel-follow after progressive reveal still nudges a few pixels] → Accepted non-goal. Do not follow when the caret remains inside the inset even if markers appear.
- [Collision with `fix-visual-edit-tail-fidelity` identity wording] → Keep whitespace `height_signature` remasurement; only narrow “changed block gets a new id” to split/merge/kind/ambiguous.

## Migration Plan

Single behavior change, no persisted format or preference migration. Rollback restores exact-slice-only identity reuse and the previous `unmeasured_below` short-circuit; documents are unchanged.

## Open Questions

- Whether an insertion exactly at a block’s start (today classified as belonging to the suffix by `map_unchanged_range`) should take the containing-range path or the unchanged-shift path. Default: leave `map_unchanged_range` as-is for wholly-outside edits; only the *interior* edit path is new. Verify with a test that types at offset 0 of a mid-document paragraph.
- Exact representation of “last measured window”: the previous visible index range stored on the tab vs. probing `bounds_for_item` on neighbors. Default: neighbors plus last caret rect are enough; store a range only if tests show a gap.
