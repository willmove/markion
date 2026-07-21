## Why

In Visual Edit mode, vertical arrow navigation across a blank line (a `Whitespace` gap row between two rendered blocks) is asymmetric: pressing **Down** from a paragraph correctly lands the caret on the blank-line gap row, but pressing **Up** from a paragraph whose block above is a heading jumps to `Whitespace.source_range.end` — the **start offset of the lower block** — instead of the gap row. From the user's perspective this manifests as three related bugs:

1. Caret at the start of a paragraph whose line above is a heading → pressing Up appears to do nothing (the resolved offset is the paragraph's own start).
2. Caret in the middle of such a paragraph → pressing Up jumps to the start of the current line instead of moving up.
3. As a direct consequence, users cannot reach an existing blank line from below to type into it, even though Down from above already works.

The Markdown-editing spec's "Layout-aware Visual Edit navigation" requirement already implies that vertical navigation follows painted layout and crosses visual blocks (including blank-line gap rows) in both directions. The bug is that the implementation did not honor this contract for the Up direction.

## What Changes

- `move_visual_vertical`'s `Whitespace` shortcut branch in `src/app/editing.rs` now resolves the target offset to `block.source_range.start` for **both** Up and Down directions. Previously Up used `block.source_range.end`, which lands on the next non-whitespace block's first offset (off-by-one past the gap row).
- As a result, Up and Down are now symmetric across a blank-line gap row: each direction lands on the gap row's source offset (which is also where the gap row's `VisualProjection` anchors, so the row accepts subsequent typed text). A second Up from the gap row continues into the heading above via the existing `pending_visual_navigation` path, preserving the preferred horizontal coordinate.
- Selection variants (`Select Up`, `Select Down`) inherit the same fix because they share `move_visual_vertical`.

## Capabilities

### New Capabilities

<!-- None — this change sharpens an existing capability. -->

### Modified Capabilities

- `markdown-editing`: The "Layout-aware Visual Edit navigation" requirement is sharpened to state that vertical navigation SHALL be symmetric across blank-line (`Whitespace`) gap rows — Up from the lower block and Down from the upper block both land on the gap row's source offset, enabling the user to type into an existing blank line from either direction.

## Impact

- **Code**: `src/app/editing.rs` — `move_visual_vertical` Whitespace shortcut branch (single-line logic change plus a clarifying comment). No new functions, no new state, no new dependencies.
- **Tests**: `src/app/tests.rs` — new regression test `visual_edit_up_arrow_into_blank_line_then_heading` covering (a) Up from a paragraph lands on the `Whitespace` block, (b) typing after Up inserts at the blank line, (c) a second Up continues into the heading preserving `preferred_x`, (d) Up from a paragraph start (above is heading) lands on the gap row rather than staying put. The existing `visual_edit_down_arrow_into_blank_line_shows_caret_not_source_island` test continues to pass, confirming the Down behavior is unchanged.
- **Invariants touched**: none. The fix only changes which source offset the Whitespace shortcut resolves to; it does not alter per-version visual-block caching, snapshot registration, `pending_visual_navigation` handoff, or the source-backed input pipeline.
- **Dependencies**: none new.
- **Non-goals**: changing in-block wrapped-line navigation; changing how `Whitespace` gap rows are rendered (still the thin-caret row from `fix-visual-edit-whitespace-caret-box`); changing navigation into or out of source islands (`FrontMatter` / `Code` / `Html` / `Unsupported`), which already falls through to source-mode `previous_line_offset` / `next_line_offset` via `current_visual_navigation_snapshot` returning `None`.
