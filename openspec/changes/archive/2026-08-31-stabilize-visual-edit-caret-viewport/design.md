## Context

Visual Edit stores the caret in canonical source offsets and paints it on a virtualized `ListState`. Two layers currently move the viewport after almost every caret change:

```
pointer / keyboard / mutation
  -> move_to / select_to
  -> visual_cursor_reveal_pending = true
  -> next Visual Edit render
       if item_ix > scroll_top.item_ix:
           scroll_to(item_ix, offset 0)   // pin later row to the top
       visual_caret_follow_frames = 2
  -> after paint, follow_visual_caret_in_list
```

The pin-to-top branch was added in `fix-visual-edit-tail-fidelity` so unmeasured suffix rows (GPUI reports height 0) can be laid out. It is correct for a brand-new tail row and wrong for a click on an already-painted mid-document row: `index > top.item_ix` is true for every row below the current scroll top, including rows sitting in the middle of the viewport.

The original caret-follow spec only named keyboard, mutation, mode entry, search, and outline as reveal triggers. Pointer placement was never supposed to jump the list. That scenario later dropped out of the main `markdown-editing` spec, so this change restores a geometry-first contract and adds Typora-style document-end air.

**Data flow (caching / versioning):**

```
pointer down on a painted row
  -> source selection update (no document.version change)
  -> geometry gate reads list viewport + item/caret bounds
  -> scroll offset changes only when the caret is outside an inset
  -> visual_caret_bounds / reveal flags stay per-tab interaction state

last-line edit that grows a row
  -> replace_source_range (version++, derived caches dropped once)
  -> visual list splices the changed row
  -> geometry gate: if painted caret is below the inset, pixel-follow
  -> no extra parse or cache invalidation for the scroll itself
```

Derived `Arc<VisualBlock>` caches, highlight memoization, and the cached text handle are not on this path except when a real source edit already invalidates them.

## Goals / Non-Goals

**Goals:**

- Geometry-gate every Visual Edit caret move: if the caret is already inside the viewport plus a small inset, leave `logical_scroll_top` alone.
- When the caret is outside that inset, scroll the minimum amount needed. Pin-to-top only for unmeasured rows that cannot be revealed by bounds.
- Add a trailing, presentation-only document-end padding band (~half the current Visual Edit viewport) so the last content line can sit away from the clip and last-line clicks rarely need to move already-visible text.
- Keep off-screen keyboard, search, mode-entry, and mutation reveal working, using the same gate.
- Keep caret/scroll/reveal state independent of document-version caches.

**Non-Goals:**

- Compensating progressive-reveal glyph shift so the clicked character stays under the mouse.
- Changing source-editor, Read, or Split Preview click/scroll.
- Redesigning typewriter mode (it may still center the caret when enabled).
- Changing outline heading top-align (`navigate_to_outline_heading`); that remains an explicit navigation affordance under `tables-outline`.
- Sync scroll, Enter semantics, or VisualBlock schema changes.

## Decisions

### D1 — One geometry gate for every caret move

All caret-moving paths (`move_to`, `select_to`, mutation `after_document_changed`, search, mode entry) may still set a one-shot “consider follow” flag. The **consume** side decides whether to scroll:

```
target item ix, optional painted caret bounds
        │
        ▼
item unmeasured and below the measured window?
  yes -> pin that item (scroll_to ix, 0) then pixel-follow
  no  -> caret (or item bounds, if caret not painted yet) inside viewport inset?
           yes -> no scroll
           no  -> minimal pixel delta (follow_visual_caret_in_list)
                  or scroll_to_reveal_item when only the item is known
```

Rejected: splitting `move_to` into pointer vs keyboard. The bug is geometric, not source-based; a keyboard Down that stays on screen must also not jump.

Rejected: keeping `if index > top.item_ix { pin }` and special-casing clicks. Any later visible row would still pin.

`follow_visual_caret_in_list` already computes a minimal delta against a 2px margin. Raise that inset to about one preview line height so a caret sitting on the last visible pixel does not immediately clip when the next glyph is typed, without pulling mid-pane clicks.

### D2 — Pin-to-top is only for unmeasured rows

Use `ListState::bounds_for_item` (or an equivalent “has a measured size” check). If the item has been laid out, it is never pinned to the top solely because its index is greater than `scroll_top.item_ix`.

Unmeasured suffix rows after Enter at EOF still need the pin so GPUI can assign a height; pixel-follow then keeps the caret inside the inset. Existing tail-typing tests stay valid.

### D3 — Document-end padding is a list-level spacer, not a VisualBlock

GPUI 0.2.2 ignores list-element padding in `max_offset_for_scrollbar` (the reason tail-fidelity moved `.pt/.pb` off the `list`). End air must live **inside** the scrollable item stream.

Add one trailing spacer item after the last `VisualBlock`:

- Not a `VisualBlock`. Incremental parse, stable IDs, `document_memory`, and the per-version visual cache stay unchanged.
- Height is about half the current Visual Edit viewport (`viewport_bounds().size.height * 0.5`), with a small floor when the viewport is already known and 0 only before the first layout.
- On viewport resize, the spacer’s cached height is invalidated so the scroll extent tracks the pane.
- Pointer down on the spacer places the caret at the document end through the same geometry gate (no pin-to-top). The spacer never paints the document caret.
- Empty documents keep the existing placeholder surface; the spacer is omitted or zero until there is at least one visual row.

Rejected: padding the last content row. That contaminates last-row measurement, splice identity, and hit-testing.

Rejected: a fixed pixel band (e.g. 240px). Half-viewport matches Typora-class last-line comfort across window sizes.

The wrapper `.pt(9).pb(9)` from tail-fidelity stays: that is pane chrome, not document-end air.

### D4 — Reveal flags stay ephemeral

`visual_cursor_reveal_pending`, `visual_caret_follow_frames`, and `visual_caret_bounds` remain per-tab interaction state. Updating them MUST NOT increment `MarkdownDocument.version()` or drop derived caches. Manual wheel/scrollbar scrolling still must not snap back unless a later caret move is actually outside the inset.

### D5 — Outline top-align stays a separate path

`navigate_to_outline_heading` currently pins the heading to the top on purpose. That contract lives in `tables-outline` and is out of scope. Search continues to `scroll_to_reveal_item` for off-screen matches; if the match is already inside the inset, the geometry gate is a no-op.

## Risks / Trade-offs

- [Unmeasured-row detection is wrong and mid-document clicks still pin] → Gate on measured bounds, not `index > top.item_ix`. Add a rendered-window test that scrolls to a mid-list row, clicks a later visible row, and asserts `logical_scroll_top` is unchanged.
- [Half-viewport spacer makes short documents suddenly scrollable] → Intended. Users can rest the last line near mid-pane. Scrollbar thumb math must include the spacer; existing pane-scrollbar tests need a Visual Edit case with the extra item.
- [Spacer item count drifts from `visual_list_blocks.len()`] → Centralize list length as `blocks.len() + spacer` in `sync_visual_list` and the row processor. Block-index helpers keep using the block slice only.
- [Resize leaves a stale spacer height and a wrong max scroll] → Invalidate/re-measure the spacer item when viewport height changes by more than a small epsilon.
- [Pixel-follow after progressive reveal still nudges the viewport] → Accept a few pixels when the caret actually crosses the inset (last line / clip). Do not follow when the caret remains inside the inset even if markers appear.
- [Typewriter mode still jumps on click] → Accepted non-goal; only runs when the preference is on.

## Migration Plan

Single behavior change, no persisted format or preference migration. Rollback removes the geometry gate and the spacer item; documents are unchanged.

## Open Questions

- Exact inset: one `preview_row_line_height` versus a fixed 8–12px. Decide during implementation from the last-line typing test; spec only requires a small inset, not a pixel value.
- Whether a viewport shorter than two line heights should shrink the spacer toward zero so a tiny pane does not become mostly padding. Default: still use half viewport; revisit only if the tiny-pane test feels unusable.
