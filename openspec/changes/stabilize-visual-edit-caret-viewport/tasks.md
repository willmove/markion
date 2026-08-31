## 1. Geometry gate

- [x] 1.1 Extract a pure helper that, given Visual Edit viewport bounds, an optional painted caret, optional measured item bounds, and a small inset, returns whether a scroll is needed and (if so) the minimal pixel delta — no document or cache access
- [x] 1.2 Unit-test the helper: caret inside the inset → no scroll; caret above/below the inset → only the overflowing delta; missing caret but measured item fully inside → no scroll; unmeasured item below the measured window → pin, not a guessed pixel delta
- [x] 1.3 Decide the inset from the last-line typing fixture (prefer one `preview_row_line_height`, fall back to ~8px) and record the choice in the change folder notes

## 2. Stop pinning visible rows

- [x] 2.1 Replace the `index > top.item_ix` pin in `src/app/root_view.rs` with the geometry gate: pin only when `bounds_for_item` is missing and the item is below the measured window; otherwise no-op or apply the helper's minimal delta
- [x] 2.2 Keep `visual_cursor_reveal_pending` / `visual_caret_follow_frames` as one-shot interaction flags, but consume them through the gate so an in-inset caret never changes `logical_scroll_top`
- [x] 2.3 Tighten `follow_visual_caret_in_list` to the same inset and skip follow when the painted caret is already inside it
- [x] 2.4 Confirm `move_to` / `select_to` / `after_document_changed` still request a consider-follow, without adding a pointer-vs-keyboard split, and without touching document-version caches

## 3. Document-end padding band

- [x] 3.1 Add a trailing list-level spacer item after the last `VisualBlock` (not a `VisualBlock`): height ≈ half the current Visual Edit viewport, omitted or zero for an empty document / pre-layout frame
- [x] 3.2 Centralize Visual Edit list length as `blocks.len() + spacer` in `sync_visual_list` and the row processor so block-index helpers keep using the block slice only
- [x] 3.3 Invalidate the spacer's cached height when the viewport height changes so scrollbar extent tracks pane resize
- [x] 3.4 Map a primary click on the spacer to the document-end source offset through the same geometry gate (no pin-to-top); the spacer never paints the document caret
- [x] 3.5 Assert the spacer does not appear in `MarkdownDocument.text`, `visual_blocks_shared()`, dirty state, or other derived caches

## 4. Tests

- [x] 4.1 Rendered-window test: long Visual Edit document, scroll so a mid-list row is at the top, click a later fully visible row, assert caret moves to the click source and `logical_scroll_top` is unchanged
- [x] 4.2 Rendered-window test: click a visible lower (not last) row and assert that row is not pinned to the viewport top
- [x] 4.3 Keep / adjust `visual_edit_tail_typing_stays_visible_at_the_viewport_bottom` so last-line Enter/typing still keeps the caret inside the inset without requiring a mid-document pin
- [x] 4.4 Test that keyboard or search navigation to an off-screen Visual Edit row still reveals that row, and that a following wheel-style scroll is not snapped back
- [x] 4.5 Test document-end padding: scroll extent is larger than content height by about half the viewport; clicking the spacer places the caret at EOF without pinning a content row; document version and visual-block `Arc` identity are unchanged

## 5. Validation

- [x] 5.1 `cargo test --workspace` passes with the new and updated tests
- [x] 5.2 `openspec validate stabilize-visual-edit-caret-viewport` passes
