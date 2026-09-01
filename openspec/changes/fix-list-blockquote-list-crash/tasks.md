## 1. Reproduce and Lock the Failure

- [x] 1.1 Add a parser regression for the minimal list → blockquote → list fixture that asserts destination ownership, authored ordering, and disjoint source ranges; run it to confirm the current implementation fails.
- [x] 1.2 Add a pure Visual Edit regression that derives the minimal fixture and asserts non-panicking, complete, monotonic, in-bounds UTF-8 source coverage; run it to confirm the current implementation fails.

## 2. Correct Parser Ownership

- [x] 2.1 Add an explicit destination to `ListItemDraft` and route every eager and ordinary item flush through that captured destination.
- [x] 2.2 Move nested-block boundary state into its owning list draft and record list-nested blockquote starts alongside code, table, and HTML starts.
- [x] 2.3 Add and pass parser variants for UTF-8, CRLF, ordered/task items, siblings, quote-contained lists, nested lists, and paragraph-only nested quotes.

## 3. Harden Visual Projection

- [x] 3.1 Validate preview, quote-group, and derived visual ranges before any string slicing; omit invalid semantic leaves so existing gap coverage provides a conservative source-backed fallback.
- [x] 3.2 Add and pass a deliberately malformed reversed/out-of-bounds/non-boundary range regression proving projection does not panic and preserves complete canonical coverage.
- [x] 3.3 Run focused Visual Edit quote/list and incremental-versus-full tests, fixing regressions without weakening range assertions or cache behavior.

## 4. End-to-End Validation

- [x] 4.1 Run formatting checks, focused parser/projection tests, and `cargo test --workspace`.
- [x] 4.2 Launch the built application with the original affected note/session and verify it remains running without a panic or new WER crash.
- [x] 4.3 Validate the OpenSpec change and record all completed tasks.
