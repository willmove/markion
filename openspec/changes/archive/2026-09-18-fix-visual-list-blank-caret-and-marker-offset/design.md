## Context

See proposal.md — Why. Key current-state facts the design builds on:

- pulldown-cmark's `End(Item)` source range for the last item of an unquoted list swallows the trailing blank lines, and `build_visual_blocks` projects that whole range as one `ListItem` visual row. `append_list_whitespace_runs` then renders each blank line as a newline run inside that row, so the row's indent (per-level `ml` plus the 22 px marker column) applies to the blank-line carets.
- Quoted lists already behave correctly: the quote group ends at the quote's last content line, so trailing blanks fall out of the group and the coverage loop emits unquoted `Whitespace` gap rows rendered full-width at the left margin (`visual_block_content_view` → `Whitespace` arm, `visual_whitespace_caret_element`, `whitespace_*` line mapping).
- Paragraph/heading rows keep their own line separator inside the block range but do not paint it as a run; blank lines after them are gap rows. The item-tail runs currently paint the separator plus every blank line, with only the final separator before a following block excluded.
- The marker column is a `flex_none` div with `min_w(22)` that stays mounted with an empty glyph while the structural prefix is revealed, offsetting the revealed raw text.

## Goals / Non-Goals

**Goals:** blank lines around unquoted lists get the same geometry as blank lines elsewhere (full-width rows, caret at the left margin); revealed list prefixes stop inheriting the marker-column offset; existing row counts, caret ownership, pointer/IME/undo round-trips, and per-version `Arc` caching stay intact.

**Non-Goals:** no change to quoted-list paths, quote gaps, Enter/Backspace structural edits, source mutation helpers, or the virtual list splice/identity model beyond what the added gap rows already exercise.

## Decisions

1. **Trim the unquoted item range at block-build time, not at parse time.** After the nested-list partition in `build_visual_blocks`, cut each unquoted `ListItem` leaf's `source_range.end` back to the end of its last non-whitespace line, keeping that line's separator (including CRLF). The coverage loop then fills the tail with ordinary `Whitespace` gap rows. Parse-level trimming was rejected: `PreviewBlock::ListItem` ranges also feed preview selection/export paths, and the visual layer already owns the row-attribution concern. Rendering-level splitting of the item's text flow was rejected as invasive: the caret geometry comes from the single `VisualEditableText` projection inside the indented flex row, and splitting projections across two elements would fork pointer/IME mapping.
2. **Match the paragraph run convention; delete the list-tail run pass.** With the trimmed range, route unquoted list items through `append_trailing_horizontal_whitespace_run` like every other block: only trailing horizontal whitespace on the content line stays a run; separators are never painted by the item. `append_list_whitespace_runs` becomes dead and is removed. Alternative kept-open door: unconditional "drop final separator" inside the old helper — rejected because it leaves two nearly identical helpers and the EOF-row accounting split across them.
3. **Extend the EOF-row rule to unquoted items.** `needs_eof_row` currently fires for Paragraph/Heading (and quoted items). Unquoted `ListItem` must join: after trimming, an item ending at EOF with a trailing newline otherwise loses its empty insertion row, changing caret-at-EOF behavior versus today. When the blank tail already covers EOF (`covered_until < text.len()` gap), the rule is inert because the gap provides the row.
4. **Gate the marker column on reveal in the view.** In the list-item arm of `visual_block_content_view`, mount the `min_w(22)` marker-column child only while the prefix is hidden. Checkbox hit targets already exist only while the prefix is hidden (`clickable_checkbox` is false when revealed), so no interaction regresses. `visual_level` collapsing to 1 while revealed stays as-is.
5. **Loose-list inter-item blanks become gap rows too.** The trim applies to every unquoted item, so a loose list's separator blank line moves from inside the previous item's indented row to a full-width gap row. This is intentional (consistent geometry for all authored blank lines) and only changes presentation, not parsing or mutations. Counted rows stay identical: the item stops painting the blank, the gap paints exactly that line.

## Risks / Trade-offs

- [Block-count churn between versions] → Whitespace gap rows are not new (paragraph gaps, quote tails); the virtual list already splices them. GPUI tests from `fix-visual-list-whitespace-lines` assert row movement and source round-trips, not owner kinds, and are re-run.
- [Caret ownership edge at the trimmed boundary] → the item keeps its own line separator, so a caret before that separator stays inside the item (end-of-content row); the first blank-line offset lands in the gap, mirroring the paragraph convention locked by existing gap tests.
- [Empty-item reveal with no runs] → empty items reveal via the `owns_caret && editable_runs.is_empty()` branch already covered by existing empty-item tests; marker-column removal only shifts the text left.
- [Two active changes touching list tails] → `fix-visual-list-whitespace-lines` (implemented, unarchived) keeps its observable guarantees under gap-row ownership; its projection-level tests are reworked in this change to assert the same guarantees through the gap rows. Archive order does not matter because neither delta contradicts the other's requirement text.

## Migration Plan

Presentation-only; no document migration. Revert is the scoped diff (trim step, run-helper routing, EOF-row condition, marker-column gate) plus the test updates.
