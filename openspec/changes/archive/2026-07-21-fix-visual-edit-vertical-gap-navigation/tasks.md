## 1. Make vertical navigation symmetric across blank-line gap rows

- [x] 1.1 In `src/app/editing.rs::move_visual_vertical`, locate the `Whitespace` shortcut branch (around `src/app/editing.rs:1422-1436`) where the cross-block fallback lands on an adjacent `VisualBlockKind::Whitespace` row. Replace the per-direction target selection `match direction { Up => block.source_range.end, Down => block.source_range.start }` with a single `let target = block.source_range.start;` so both Up and Down resolve to the gap row's source offset. Add a clarifying comment explaining the previous `.end` choice landed on the lower block's first offset (off-by-one past the gap row), which made Up from a paragraph look like nothing happened.
- [x] 1.2 Verify the second-Up path (caret already inside the gap row, pressing Up again) continues to take the `pending_visual_navigation` handoff into the rendered block above, since the gap row's snapshot has a single painted line and `adjacent_line` is `None`. No code change required; confirm via the regression test in 2.1.

## 2. Tests

- [x] 2.1 Add an integration test `visual_edit_up_arrow_into_blank_line_then_heading` in `src/app/tests.rs` against `"### heading\n\nparagraph"` that covers all three reported scenarios: (a) from a caret at paragraph mid (`offset 16`), pressing Up lands at `offset 12` on the `Whitespace` block with `visual_input_bounds.is_some()`; (b) simulating input "x" after that Up mutates the document to `"### heading\nx\nparagraph"`; (c) a second Up from the gap row continues into the `Heading` block with `visual_preferred_x` retained; plus a separate window verifying Up from paragraph start (`offset 13`) also lands at `offset 12` rather than staying at `13`.
- [x] 2.2 Confirm the existing `visual_edit_down_arrow_into_blank_line_shows_caret_not_source_island` test still passes, locking the unchanged Down-direction behavior.

## 3. Validation

- [x] 3.1 Run `cargo test --workspace` and ensure every crate's suite passes (0 failures).
- [x] 3.2 Run `cargo clippy --all-targets` and confirm no new warnings are introduced by the change in `src/app/editing.rs` or the new test in `src/app/tests.rs`.
- [x] 3.3 Run `openspec validate fix-visual-edit-vertical-gap-navigation` and resolve any reported inconsistencies.
