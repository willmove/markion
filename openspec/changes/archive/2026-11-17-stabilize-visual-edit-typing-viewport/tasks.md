## 1. In-place identity reuse

- [x] 1.1 Add a GPUI-free containing-range mapper beside `map_unchanged_range` that expands a block range only when the pending edit is wholly interior; overlapping boundary edits return `None`
- [x] 1.2 Extend `reconcile_visual_block_ids` with a second pass: after exact-slice reuse, copy the old id onto the unique new block whose `source_range` equals the mapped containing range and whose `kind` matches
- [x] 1.3 Unit-test in-place paragraph/list/heading typing: successor keeps the old id; later unchanged blocks still keep theirs; `visual_block_splice` is a no-op for that row
- [x] 1.4 Unit-test split (Enter), merge, and kind change (`# ` / `- `): affected successors get new ids; occurrence-equal unedited paragraphs are not stolen
- [x] 1.5 Confirm a whitespace row that keeps its id still splices when `height_signature` changes (tail-fidelity remasurement)

## 2. Geometry gate for temporarily unmeasured visible rows

- [x] 2.1 Reorder `visual_caret_scroll_action` so a positive-height caret or item rect is tested against the inset before `PinItem`; pin only when both are missing and the item sits below the previously measured window
- [x] 2.2 Unit-test the helper: last caret inside the inset with `item_bounds = None` → no scroll; unmeasured index below the measured window and no caret → `PinItem`; measured item inside inset → no scroll
- [x] 2.3 Stop `after_document_changed` from clearing `visual_caret_bounds`; keep `visual_cursor_reveal_pending` as a consider-follow flag consumed through the gate
- [x] 2.4 Confirm `visual_caret_follow_frames` still pixel-follows clipped last-line growth and is a no-op when the new caret stays inside the inset; no document-version cache access on this path

## 3. Rendered Visual Edit tests

- [x] 3.1 Rendered-window test: long document, scroll so a mid-list row is at the top, type in a later fully visible row, assert `logical_scroll_top` is unchanged and that row is not pinned
- [x] 3.2 Rendered-window test: IME-style `replace_text_in_range` (or equivalent composition replacement) in a visible mid-document row does not pin and does not change `logical_scroll_top` except for inset clip follow
- [x] 3.3 Rendered-window test: Enter in a visible mid-document paragraph whose resulting caret stays inside the inset does not pin the successor to the top
- [x] 3.4 Keep `visual_edit_tail_typing_stays_visible_at_the_viewport_bottom` and the existing click-does-not-scroll tests green
- [x] 3.5 Assert in-place typing does not increment extra parses or drop the per-version visual-block cache beyond the single rebuild already required by `version++`

## 4. Validation

- [x] 4.1 `cargo test --workspace` passes with the new and updated tests
- [x] 4.2 `openspec validate stabilize-visual-edit-typing-viewport` passes
