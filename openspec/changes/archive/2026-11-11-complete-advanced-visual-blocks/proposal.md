## Why

Section 18 P3 remains incomplete after P1: complex containers cannot be manipulated as source units, cells lack rectangular operations and final-cell Tab, and code cannot fold. Existing image size/alignment/editing should remain usable and verified.

## What Changes

- Extend exact source units to nested list subtrees, complete quote groups and contiguous block ranges; allow safe container splitting and subtree indent/outdent/reordering with shared pointer/keyboard operations.
- Add rectangular/row/column cell selection, TSV/HTML copy, matrix paste and clear; append a row with Tab in the final cell.
- Patch ordinary cell text locally rather than reformatting the whole table on each character.
- Add session-only ordinary code-block folding with a visible summary and reliable expansion when editing/navigating.
- Verify existing image source editing, width, alignment and drag resize through the same stale-safe/undo contracts.
- Preserve canonical Markdown, shared per-version caches, memoized highlighting and cache-free undo snapshots.

Non-goals: merged cells, arbitrary block trees inside GFM cells, image pixel cropping or a second document format.

## Capabilities

### New Capabilities

### Modified Capabilities
- `markdown-editing`: complete container/source-unit operations and subtree movement.
- `tables-outline`: rectangular cell commands, local cell edits and final-cell Tab row creation (replacing the previous boundary behavior).
- `code-and-math`: session-only code folding.

## Impact

Pure block/table helpers, explicit selection and transaction layer, virtualized visual rows, keyboard and drag/drop routing, localized controls, regression fixtures and desktop validation.
