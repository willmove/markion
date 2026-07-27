## 1. Reliability foundation

- [x] 1.1 Preserve existing destination permissions in atomic replacement and add cross-platform/Unix-focused filesystem tests.
- [x] 1.2 Add recovery inventory metadata and disk-relationship classification without eagerly deleting or loading unrelated snapshots.
- [x] 1.3 Replace startup all-or-nothing recovery prompting with per-entry and bulk recovery-manager actions, retaining unreadable/unselected snapshots.
- [x] 1.4 Reuse matching session-restored tabs and retain restored recovery files until durable save, successor recovery, or explicit discard, with focused lifecycle tests.

## 2. Exact block model

- [x] 2.1 Add GPUI-free slash command, block transform, block target, source unit, duplicate/delete, and reorder types/helpers.
- [x] 2.2 Implement UTF-8/CRLF-safe transformations for text, H1-H6, bulleted/numbered/task lists, quote, fenced code, divider, and table templates.
- [x] 2.3 Implement exact source-unit duplication/deletion and move before/after/up/down with separator preservation and conservative nested/overlap rejection.
- [x] 2.4 Add pure tests for every transform, slash template, stale target, nested/quote rejection, UTF-8, CRLF, leading/trailing whitespace, and undo selection results.

## 3. Slash command experience

- [x] 3.1 Add ephemeral slash-query state derived only from a collapsed Visual Edit line and clear it on mode/tab/version changes.
- [x] 3.2 Add localized filtered palette UI with pointer selection, Up/Down navigation, Enter confirmation, Escape cancellation, and empty-result presentation.
- [x] 3.3 Route confirmation through one atomic canonical mutation and add rendered GPUI tests for keyboard, pointer, cancellation/cache stability, stale ranges, and IME query input.

## 4. Block menu and reorder experience

- [x] 4.1 Add focused-row block chrome with Turn Into, Duplicate, Delete, Move Up, and Move Down operations using version/id/range validation.
- [x] 4.2 Add drag grips and before/after drop targets for proven reorderable rows; ensure drag movement is presentation-only and shares the move helper.
- [x] 4.3 Add GPUI tests for transformations, duplicate/delete, button and drag reorder equivalence, one-step undo/redo, stale events, multi-tab isolation, and conservative disabled states.

## 5. Product integration and verification

- [x] 5.1 Add every P1 string to every supported localization catalog and validate completeness.
- [x] 5.2 Update the Visual Edit support/evidence document with slash, block transformation, and reorder ownership/fallback rules.
- [x] 5.3 Run focused storage/model/GPUI tests, `cargo fmt --check`, `cargo test --workspace`, the repository quality script, and strict OpenSpec validation; resolve failures.
- [x] 5.4 Archive `complete-p1-reliability-block-editing`, verify stable spec sync, and create a separate conventional P1 commit without pushing.
