## Context

See `proposal.md` for motivation. Read mode already overlays a custom vertical scrollbar on the virtualized preview `list`. Visual Edit uses the same GPUI `ListState` pattern (`EditorTab::visual_list`) and already insets content with `PREVIEW_SCROLLBAR_SAFE_RIGHT_PADDING`, but `visual_edit_surface_view` never attaches an overlay.

Current render flow:

1. View mode selects the source editor, Visual Edit surface, or preview list.
2. Source Edit uses a `ScrollHandle` plus `pane_scrollbar_view`.
3. Read / Split Preview uses `preview_list` plus `preview_list_scrollbar_view`.
4. Visual Edit hosts `visual_list` under an IME `VisualInputElement` overlay that registers input handling and stores bounds, but deliberately inserts no pointer hitbox.

Scrolling Visual Edit therefore already works by wheel/trackpad against the list. The missing piece is a visible, draggable thumb on the same `visual_list` state. Derived Markdown caches, visual-block mapping, and document versioning are not on this path.

## Goals / Non-Goals

**Goals:**

- Overlay a Read-mode-equivalent right-side scrollbar on the Visual Edit surface when `visual_list` has a scrollable range.
- Drive overlay geometry and drag from the active tab's `visual_list`, sharing wheel/trackpad/keyboard reveal with that same list.
- Keep the overlay hit-testable above editing chrome without stealing caret, selection, or IME input.
- Leave Sync scroll inactive for Visual Edit, including after returning to Split Preview.

**Non-Goals:**

- Sharing `preview_list` and `visual_list` scroll offsets, or coupling Visual Edit with Sync scroll.
- Changing Edit / Split Preview / Read scrollbar chrome, density constants, or reserved padding.
- Adding a scrollbar auto-hide preference or platform native scrollbar.
- Changing Visual Edit virtualization, row measurement, or document mutation paths.

## Decisions

1. Reuse the existing ListState overlay, parameterized, instead of a third scrollbar implementation.

   Rationale: `preview_list_scrollbar_view` already maps viewport height, frozen drag height, thumb travel, and `set_offset_from_scrollbar`. Visual Edit is the same widget class. Parameterize element id and whether a Split Preview sync-scroll driver is marked, then call it from `visual_edit_surface_view` with `visual_list`.

   Alternative considered: duplicate the overlay as `visual_list_scrollbar_view`. That would copy drag math and make Read/Visual Edit diverge.

2. Do not mark Visual Edit drags as `PaneScrollTarget::Preview`.

   Rationale: `preview_list_scrollbar_view` currently records a Preview sync-scroll driver. `sync_scroll_is_active` is false in Visual Edit, so reconciliation is a no-op in that mode, but a leftover Preview driver can still be observed after switching to Split Preview. Visual Edit therefore uses a distinct overlay id and omits the sync-scroll driver mark (or uses a target that `mark_sync_scroll_driver` ignores).

   Alternative considered: reuse the Preview overlay unchanged because the preview pane is `hidden()` in Visual Edit. Hidden does not clear per-tab sync-scroll driver state.

3. Paint the overlay as the last child of `visual_edit_surface_view`.

   Rationale: the IME overlay is a full-surface absolute sibling. Even though `VisualInputElement` has no hitbox, stacking the thumb last matches Read mode (content then overlay) and keeps `block_mouse_except_scroll` on the thumb above any future overlay chrome. Existing right-side content padding already leaves a gutter for the thumb.

   Alternative considered: inset the IME overlay so it never covers the gutter. That would change IME bounds for no current hit-testing benefit.

4. Hide the thumb with the same empty/short-content rule as Read mode.

   Rationale: Visual Edit already swaps the list for a placeholder when there are no visual blocks. When the list exists but `max_offset_for_scrollbar` is at most one pixel, hide the overlay. No new preference or always-visible track.

   Alternative considered: always show a track. That would differ from Read mode and occupy attention on short notes.

## Risks / Trade-offs

- [Risk] The IME overlay or a later full-surface hitbox could swallow thumb drags. → Mitigation: keep the scrollbar as the last sibling with `block_mouse_except_scroll`, and verify drag on a long document.
- [Risk] Reusing Preview driver state would desynchronize Split Preview after a Visual Edit scroll. → Mitigation: never mark Preview/Editor drivers from the Visual Edit overlay; add a regression that Visual Edit scrolling does not become a preview-driven sync update.
- [Risk] Virtualized row measurement can change reported content height during a drag. → Mitigation: keep `scrollbar_drag_started` / `scrollbar_drag_ended` so thumb height stays frozen, as Read mode already does.
- [Risk] Tests cannot cheaply assert painted pixels. → Mitigation: cover ListState offset updates, document-version stability, and reserved-gutter constants; confirm painted thumb manually.

## Migration Plan

No preference, document, or cache migration. Ship as a chrome fix in the next release. Rollback is reverting the Visual Edit overlay attachment and any helper parameterization.

## Open Questions

None.
