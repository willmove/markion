## Context

See proposal.md. GFM and source offsets remain authoritative. Pure selection/transactions from P2 provide source ranges; existing visual row grips and table handles already carry immutable targets. Image width/alignment and resize exist and require regression coverage rather than a second implementation.

## Goals / Non-Goals

Goals: complete source-unit/subtree operations, quote splitting, rectangular GFM editing, local cell patches, final-cell Tab and code folding.

Non-goals: merged/rich block cells or pixel transformations that require a custom document format.

## Decisions

1. Derive complete list subtrees from ordered visual rows and enclosing list depth, and complete quote units from exact quote-group ranges. Selection expansion includes descendants. Reject destinations inside moved units; preserve source bytes with a minimal contiguous replacement. Indent/outdent applies to every owned line, preserving nested code and CRLF.
2. Converting a quote leaf to text strips only its exact quote prefixes; blank-line boundaries split surrounding quoted siblings. Keep source ownership checks and parser-derived ranges.
3. Cell rectangles resolve through exact mapped cells. Clear/matrix paste serializes one table operation with escaped pipes/newlines; ordinary typing replaces only the current field range and returns the exact caret. Final Tab appends one empty row; backward traversal at the first cell retains the previous block behavior.
4. Code folding is per-tab presentation state keyed by stable ID. The summary retains an expansion control and source payload remains canonical. Editing/navigation reveals a folded owner before positioning the caret. Fold/unfold changes neither version nor history.
5. Pointer/keyboard routes invoke the same transactions; drag previews do not mutate the source. Auto-scroll is bounded to the current visual surface and stale drags are canceled.

Data flow: captured selected units/table rectangle -> exact pure edit -> version/identity checked transaction -> one history entry -> source-derived rows. Folding and drag feedback branch only into presentation state.

## Risks / Trade-offs

- Markdown can merge adjacent containers -> deterministic separators and full-parse regression fixtures.
- Ragged/escaped-pipe cells -> shared parser and sanitizer, reject unproven mappings.
- Folding hides active input -> expand on explicit caret navigation and input.
