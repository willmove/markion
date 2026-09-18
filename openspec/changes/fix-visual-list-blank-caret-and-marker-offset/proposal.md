## Why

Visual Edit paints two wrong caret-geometry positions around unquoted lists:

1. Trailing blank lines after a list belong to the last `ListItem` visual block (pulldown-cmark's item end range swallows the blank tail), so the caret on those blank rows is painted inside the indented list row — nested-level margin plus the bullet marker column — instead of at the row start. With `- a\n    - b` followed by blank lines, the caret aligns with `b`'s text, not the left margin. Quoted lists already emit unindented `Whitespace` gap rows for the same source; the unquoted path is inconsistent.
2. When the structural prefix of a list row is revealed (caret on `-` or the space after it), the row keeps its 22 px reserved marker column even though the marker glyph is hidden, pushing the raw `-` roughly 22 px to the right of where the bullet renders unfocused.

## What Changes

- Trim each unquoted `ListItem` visual block's source range at the end of its last non-whitespace line (keeping that line's separator, matching the paragraph convention), so the coverage loop emits ordinary full-width `Whitespace` gap rows for the trailing blank lines.
- Route unquoted list items through the shared trailing-horizontal-whitespace run helper (same as paragraphs/headings) and drop the now-unused list-whitespace-tail run pass.
- Extend the EOF-row rule to unquoted list items whose last line ends with a newline, so the caret at end-of-file keeps its own empty insertion row.
- Hide the reserved marker column while a list row's structural prefix is revealed, so the revealed `-`/`1.`/`- [ ]` starts at the same left edge as the unfocused bullet/number/checkbox glyph.
- Keep quoted list items, quote gaps, and all source mutations unchanged — this is a block-attribution and presentation fix only.

Non-goals: changing Enter/Backspace list continuation semantics, normalizing authored whitespace, or reworking the virtual list cache.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `markdown-editing`: adds requirements for (a) unquoted list-item trailing blank lines rendering as unindented, individually addressable `Whitespace` rows with the caret at the row start, and (b) revealed list structural prefixes aligning with the unfocused marker's left edge. The existing whitespace-only list tail requirement (change `fix-visual-list-whitespace-lines`) remains true — lines stay individually addressable and source-backed; only their owning row and indent change.

## Impact

- `src/visual.rs`: `build_visual_blocks` (new tail-trim step, EOF-row condition), `visual_block_from_preview` (run helper routing), removal of `append_list_whitespace_runs`.
- `src/app/preview.rs`: `visual_block_content_view` list-item arm (marker column gating).
- Tests: rework the two visual.rs list-tail projection tests to assert gap-row ownership and line mapping; add regressions for the user fixture (nested list + blank tail) and the revealed-marker left edge; existing GPUI caret/Enter/pointer/undo tests keep their row-movement assertions.
- Architecture invariants preserved: blocks stay cached per document version and shared via `Arc`; no derived state is recomputed per keystroke.
