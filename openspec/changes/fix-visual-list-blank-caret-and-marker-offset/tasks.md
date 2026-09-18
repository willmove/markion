## 1. Block attribution for unquoted list tails

- [x] 1.1 In `build_visual_blocks`, after the nested-list partition, trim each unquoted `ListItem` leaf range to the end of its last non-whitespace line (keeping that line's separator, CRLF-safe); extend `needs_eof_row` to unquoted list items ending with a newline. Verify with a new unit test asserting the gap-row layout for the user fixture (nested list + blank tail), a loose list, a mid-document tail, and EOF variants.
- [x] 1.2 Route unquoted list items through `append_trailing_horizontal_whitespace_run`, delete `append_list_whitespace_runs`, and rework `list_blank_line_projection_is_source_backed_and_cached` / `list_whitespace_tail_preserves_lines_and_source_positions` to assert the same guarantees (line preservation, exact source mapping, final-separator ownership) through `Whitespace` rows and `whitespace_source_at_line` round trips.

## 2. Revealed marker alignment

- [x] 2.1 In `visual_block_content_view`'s list-item arm, mount the `min_w(22)` marker-column child only while the prefix is hidden. Verify with a GPUI regression comparing the painted caret/reveal left edge on the marker against the unfocused marker row's left edge, plus existing task-checkbox and empty-item tests.

## 3. Integration verification

- [x] 3.1 Run `cargo test` for the root package (unit + GPUI suites), `cargo fmt`, and `openspec validate fix-visual-list-blank-caret-and-marker-offset`; confirm no behavior regressions in Enter/Backspace, pointer placement, IME, and undo around lists.
