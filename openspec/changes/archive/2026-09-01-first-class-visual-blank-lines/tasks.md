## 1. Empty-paragraph height for Whitespace rows

- [x] 1.1 Change `whitespace_row_height` (and Y-mapping helpers that divide by the old 12px constant) to use `DocumentTypographyMetrics.paragraph_line_height` instead of `WHITESPACE_ROW_LINE_HEIGHT`. Keep the one-line floor and `WHITESPACE_ROW_MAX_LINES` cap. `visual_block_splice` continues to map newline-count `height_signature` through the live px helper so gap growth still remasures; typography changes remasure without bumping document version.
- [x] 1.2 Update unit tests that assert 12px (`whitespace_row_height_grows_with_blank_lines_without_the_old_cap`, caret-Y helpers). Add a test that default rendered size yields body paragraph line height per covered newline, and that a larger rendered font size increases the px height without mutating source.

## 2. Landing offset stays inside the authored blank line

- [x] 2.1 Adjust `whitespace_source_at_line` / `whitespace_source_at_y` so line *i* maps to the *i*-th newline offset inside `source_range`, clamped to `[start, end)` when a following content block owns `end`. A one-newline heading-to-heading gap lands at `source_range.start`, not the next heading’s first content byte.
- [x] 2.2 Add pure tests for `"## [Unreleased]\n\n## [16.1.7]"` (and `Para 1\n\nPara 2`): Y at the top of the gap row resolves to the separator newline; it does not resolve to the following block’s start.

## 3. Click-to-enter without inserting a newline

- [x] 3.1 Always attach the whitespace editor surface (I-beam, Y→source, `visual-whitespace-gap` hook) even when the row does not own the caret. Paint the insertion caret only when `owns_caret`. Do not insert `\n` on click.
- [x] 3.2 Replace `visual_edit_heading_to_heading_gap_click_is_passive` / `visual_edit_heading_to_paragraph_gap_click_is_passive` with tests that click the gap: selection moves into the whitespace range, document version / dirty / undo / Arc visual-block identity stay unchanged, and a caret is presented.
- [x] 3.3 Add a GPUI test: click the blank line between `## [Unreleased]` and `## [16.1.7]`, type `notes` — source becomes a paragraph between the headings, the next heading’s first character is untouched, and no extra blank line is inserted.

## 4. Arrow-to-enter

- [x] 4.1 Remove the skip-consecutive-Whitespace loop in `move_visual_vertical`. Up/Down (and Select Up/Down) land on the adjacent `Whitespace` row at the landing offset from 2.1. A further press leaves the row (or walks another painted line inside a multi-line gap). Keep `preferred_x` and `pending_visual_navigation` for virtualized targets. Preserve per-version visual-block cache identity (navigation is selection-only).
- [x] 4.2 Invert `visual_edit_down_arrow_skips_blank_line_gap_to_next_block` and `visual_edit_up_arrow_skips_blank_line_gap_to_heading`: one Down/Up parks on the gap; typing inserts at that newline; a second Up/Down continues into the far-side block with `preferred_x` retained. Cover Up from paragraph start landing on the gap, not the heading.
- [x] 4.3 Keep heading-Enter insertion-line tests passing (`visual_edit_heading_enter_activates_insertion_line_for_typing` and whitespace caret-not-island). Confirm leading/trailing blank lines at the document edge are still reachable with arrows.

## 5. Coverage matrix

- [x] 5.1 Update `docs/visual-editing-quality.md` blank-line row: empty-paragraph-height Visual Edit row; click and arrows enter; caret on existing source; click is not a mutation. Class remains rendered WYSIWYG.

## 6. Validation

- [x] 6.1 Run `cargo test --workspace` (click/arrow into a blank line MUST NOT rebuild derived caches; typing still goes through canonical source).
- [x] 6.2 Run `openspec validate first-class-visual-blank-lines` and resolve any reported inconsistencies.
