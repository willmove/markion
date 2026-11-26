## Context

The generic grip and the cell editing menu share action state, layout and callbacks. The cell editing menu already scopes commands to the table and its exact cell.

## Goals / Non-Goals

Keep whole-table actions compact and label the commands whose insertion or wrapping scope needs to be explicit. Preserve normal block grip and cell context behavior.

## Decisions

- Specialize the grip catalog using the captured table context. Offer Wrap in Quote only for a top-level unquoted table with exact transform ownership.
- Use the shared insertion flyout with the existing checked divider and table actions. Keep the paragraph action at the root. Group labeled rows through the same layout calculation for painting and keyboard scrolling.
- Reuse scoped clipboard, move, delete, wrap and undo behavior. Menu interaction leaves per-version derivations intact until an edit is applied.

## Risks / Trade-offs

A hover or keyboard callback from a closed flyout must not activate its child. Reuse live menu membership and target checks and exercise pointer and keyboard sequencing.
