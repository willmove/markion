# Implementation Notes — fix-visual-edit-tail-fidelity

## Task 2.4 — tail whitespace identity across growth

`reconcile_visual_block_ids` (`src/source_mapped.rs`) reuses a block id only when
the old source range maps to a range with **identical slice content** and the
rebuilt slice contains a block with **exactly that range**. When the trailing
whitespace grows, the rebuilt tail block's range is longer, so no candidate
matches and the tail block gets a **fresh id** — the row is re-spliced and
re-measured through id churn already. The `height_signature` field is therefore
defense-in-depth on two levels:

1. `visual_block_splice` compares `(id, height_signature → rendered height)`, so
   any future path that keeps a stable id on a height-changed whitespace row
   still forces a re-measure (covered by
   `visual_block_splice_keys_height_mutable_rows_on_signature`).
2. Signature participates in struct equality, so `reconcile_visual_block_ids`'s
   `shifted == *candidate` adoption check refuses to adopt an id across a
   height change even if ranges and slices somehow matched.

Integration coverage: `visual_block_splice_remeasures_tail_whitespace_when_it_grows`.

## Task 3.4 — design open questions, resolved with evidence

- **12px constant vs typography-derived per-line height**: kept the literal
  12px per blank line (`WHITESPACE_ROW_LINE_HEIGHT`). The compact blank-line
  representation is the established visual language for inter-block spacing;
  changing it to the full preview row line height would inflate every
  document's rendered gaps, not just pathological tails. Fidelity (no cap)
  was the requirement; scale was not.
- **Preview list and the height signature**: not applicable. The preview list
  renders only `PreviewBlock`s — blank source lines never become preview rows
  (gap blocks exist only in the visual block stream), so the preview list has
  no height-mutable rows. The preview list did get the D3 padding restructure
  (same latent bottom-clipping shortfall).

## Task 6 — tail caret follow (2026-08-29)

Row-height fidelity made the trailing whitespace *row* grow, but two remaining seams still made tail editing look like a no-op:

1. The whitespace caret was painted at the row origin (empty projection → display 0). Each extra Enter grew the row downward while the caret stayed put.
2. `scroll_to_reveal_item` only reveals the *block* and uses pre-layout / stale heights. A last paragraph or whitespace row that was already visible could grow below the viewport; typed characters and the caret were clipped.

Fix: map whitespace caret/clicks by covered-newline line index (12px/line), and after a reveal, pixel-follow `visual_caret_bounds` into the list viewport for two frames.

## Task 3.3 — manual verification (debug build, 2026-08-28)

Launched `target/debug/markion.exe` with `工期保障 - 副本.md` (228 lines, ~45
tail blank/continuation lines), switched to Visual Edit (Ctrl+Alt+4), jumped to
document end (Ctrl+End), pressed Enter 5 times:

- All 5 Enters wrote to the source (autosave: 228 → 233 lines) — the "only the
  first Enter works" symptom is gone.
- The tail whitespace region renders tall (the pre-existing ~45 blank lines now
  occupy ~540px instead of the old 72px cap) and the thin caret stays visible
  inside it.
- The last text lines render fully (no top clipping) with the pane scrolled to
  the bottom; the scrollbar thumb reaches the bottom of its track.
- Ctrl+Z ×5 restored the file to 228 lines (undo removes tail growth
  symmetrically).

## Task 4.4 — repro procedure for the in-memory heading duplication

If the outline ever shows duplicated headings again:

1. **Do not undo or close.** The duplication lives only in the in-memory text.
2. Select all in the editing surface (Ctrl+A), copy (Ctrl+C), and paste into a
   scratch file — this snapshot is the corrupted text.
3. Collect the log window: `%LOCALAPPDATA%\Markion\Logs\markion.<date>.log`,
   filtered to `markion::document` / `markion::editing` debug lines. Each
   canonical mutation logs document version, edit range, and old/new lengths;
   the tagged `op` lines identify the writing path that produced the
   duplication.
4. The outline panel mirrors the document exactly (verified during diagnosis);
   duplicated outline rows prove duplicated source lines, and the logs prove
   which operation wrote them.
