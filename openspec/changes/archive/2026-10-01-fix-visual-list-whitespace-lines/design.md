## Context

See proposal.md. The horizontal-whitespace helper appends spaces from a later blank line to the item's last inline run. The newline helper scans only a contiguous CR/LF suffix and starts after those spaces, so earlier newlines disappear. Missing source positions then collapse onto one display boundary.

## Goals / Non-Goals

Goals: recover each unrepresented whitespace suffix byte in source order, preserving line geometry and existing separator ownership.
Non-goals: change the canonical Enter mutation, normalize whitespace, or redesign virtual list caching.

## Decisions

For unquoted lists, replace the two independent tail helpers with one ordered whitespace-tail pass after the last represented inline content, reveal-group syntax or structural prefix. Append horizontal whitespace and normalized line-ending runs with exact source ranges. Keep the existing rule omitting the final separator before another block. Leave quoted-row handling in its existing path.

Data flow stays source mutation/version -> cached visual runs -> interaction projection -> GPUI text layout/caret. Do not patch caret coordinates or force cache invalidation: the missing source-backed rows must exist for pointer placement and IME as well as paint.

## Risks / Trade-offs

- Duplicating inline syntax or hard breaks -> start after represented content and complete reveal groups, and cover formatted/link tails.
- Extra line before a following block -> preserve the final-separator rule with dedicated assertions.
- Tests only validating text -> assert GPUI painted caret movement, pointer mapping, typing and undo in addition to model projections.

## Migration Plan

No document migration; authored spaces, tabs and line endings are preserved. Revert the scoped projection change if necessary.
