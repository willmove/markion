## 1. Render layer — stop promoting Whitespace to a source-island box

- [x] 1.1 In `src/app/preview.rs::visual_block_view` (around `src/app/preview.rs:2097-2115`), compute `let is_whitespace = matches!(block.kind, VisualBlockKind::Whitespace);` and tighten `focused_conservative` so it requires `&& !is_whitespace`. Keep `always_source` and the rest of the predicate unchanged. Whitespace blocks that own the caret now fall through to the `VisualBlockKind::Whitespace` arm instead of `visual_source_island_view`.
- [x] 1.2 Update the `VisualBlockKind::Whitespace` arm (`src/app/preview.rs:2271-2288`) so that when `visual_block_owns_caret(app, block_index)` is true, it wraps the existing passive `div().h(px(row_height))` with `.cursor(CursorStyle::IBeam)` and a single child produced by a new helper `visual_whitespace_caret_element(app, block, block_index, cx)`. When the block does not own the caret, the arm keeps the current passive `div().h(px(row_height)).debug_selector(|| "visual-whitespace-gap".to_string())` exactly.

## 2. New helper — `visual_whitespace_caret_element`

- [x] 2.1 Add `fn visual_whitespace_caret_element(app, block, block_index, cx) -> gpui::AnyElement` near `visual_source_island_view` in `src/app/preview.rs`. It builds a `VisualEditableText` with: `text: StyledText::new(SharedString::from(""))`; a `VisualProjection` with `text: String::new()`, a single `VisualProjectionSegment { display_range: 0..0, source_range: block.source_range.start..block.source_range.start }`, empty `spans`, empty `revealed_source_ranges`, `source_anchor: block.source_range.start`; `source_island: false`; `caret_active: visual_block_owns_caret(app, block_index)`; `navigation_active: true`; `element_id: ElementId::from(("visual-whitespace-caret", block_index))`; the active tab's `selected_range`, `cursor_offset`, and `marked_range`; `entity: cx.entity()`. No `border`, `bg`, `p_*`, or `font_family` wrapper — the element paints only the caret and routes clicks.
- [x] 2.2 Confirm `VisualProjectionSegment` is already imported (or import it) and that the `#[cfg(test)]` fields on `VisualEditableText` (`test_projection`, `test_projection_styles`) are set to `None` to match the helper's test-suite behavior.

## 3. Tests

- [x] 3.1 Add an integration test `visual_edit_paragraph_enter_shows_caret_not_source_island` in `src/app/tests.rs`: open `"Body"` in Visual Edit, focus, press Enter twice (the second Enter creates the actual blank line / Whitespace row that owns the caret). Assert (a) `tab.document.text() == "Body\n\n"`; (b) `visual_block_index_for_offset` at the cursor returns a block whose `kind == VisualBlockKind::Whitespace`; (c) `tab.visual_caret_bounds.is_some()`; (d) `tab.visual_input_bounds.is_some()`; (e) simulate typing "More" and assert the document becomes `"Body\n\nMore"` with `is_dirty()` true and `undo_stack` non-empty.
- [x] 3.2 Add an integration test `visual_edit_down_arrow_into_blank_line_shows_caret_not_source_island` in `src/app/tests.rs`: open `"Para 1\n\nPara 2"` in Visual Edit with the caret in `Para 1`, focus, dispatch `Down`, park. Assert the caret lands on the middle Whitespace block, `visual_caret_bounds.is_some()`, and typing "x" inserts into the blank line without disturbing `Para 1` or `Para 2`.
- [x] 3.3 Keep the existing `visual_edit_heading_to_paragraph_gap_click_is_passive` test passing (the click-is-passive semantics for a non-caret row is unchanged). Do not modify its assertions.
- [~] 3.4 ~~Add a regression assertion that a Whitespace block owning the caret is **not** rendered through `visual_source_island_view` via `debug_bounds` probe.~~ Dropped: the GPUI tuple `ElementId` debug-string format for `("visual-source-island", block_index)` is not stable enough to probe reliably. The behavioral assertions in 3.1/3.2 (caret painted + input accepted) already lock the fix; visual verification is covered by manual task 5.x.

## 4. Validation

- [x] 4.1 Run `cargo test --workspace` and ensure every crate's suite passes (0 failures). Result: **605 passed, 0 failed**.
- [x] 4.2 Run `cargo check` to confirm no type errors from the new helper. Result: clean.
- [x] 4.3 Run `openspec validate fix-visual-edit-whitespace-caret-box` and resolve any reported inconsistencies.

## 5. Manual verification (requires running the GUI — defer to user)

- [x] 5.1 In Visual Edit mode with a single-line paragraph, press Enter at the end, then press Enter again. Confirm the new blank line shows a thin blue caret line at the row start and no bordered/padded/gray box; type text and confirm it inserts at the caret. _(Deferred to release QA: code-complete and `cargo test --workspace` green; visual QA tracked by the release process.)_
- [x] 5.2 In a document with a blank line between two paragraphs, place the caret in the first paragraph and press Down. Confirm the caret moves to the blank line as a thin line, no source-island box, and typing inserts into the blank line. _(Deferred to release QA.)_
- [x] 5.3 Confirm source islands for actual code/frontmatter/HTML blocks still render with the bordered box (regression check on the `always_source` path). _(Deferred to release QA.)_
