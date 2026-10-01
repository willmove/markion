## Context

See proposal.md. Source Enter transitions already have tests; new-row geometry does not. Derived blocks and GPUI virtual rows must agree after the mutation.

## Goals / Non-Goals

Verify real row/caret positions and repair the first failing seam. No second editor document or synthetic persisted list state.

## Decisions

Data flow: source mutation/version -> cached preview/visual blocks -> stable row reconciliation -> current source projection -> painted caret and IME bounds. Use parser/projection evidence before changing layout. Preserve exact source ranges and per-version caches rather than forcing full reparses after every key. Compare fresh derivation and edited-document derivation in tests. Assert the new row/caret belongs to the new item, not merely that a caret exists.

## Risks / Trade-offs

- EOF, trailing blank lines and CRLF can shift ownership -> exercise each in regression tests.
- Layout changes can affect keyboard/IME -> type into the newly created row and verify undo and caret bounds.

## Migration Plan

No data migration. Revert the scoped code change to roll back.

## Root cause and chosen repair

Pulldown list containers include trailing blank lines without inline events.
Preserve those bytes as newline runs in the existing list projection. Between
blocks, the final separator belongs to the following row; at EOF all trailing
line endings need visible rows. This keeps existing source-range ownership and
structural Enter/Backspace behavior. Mixed layouts add an empty-line source
anchor so links and scripts do not leave the painted caret on an older row.
