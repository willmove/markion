## 1. Region splitting fix

- [x] 1.1 In `src/source_mapped.rs`, make `starts_with_continuation` return true for any line whose raw text begins with a space or tab (keep existing trimmed-marker checks for `>`|`|`|`<`|bullets|ordered markers).
- [x] 1.2 In `split_regions`, insert the fence-opening boundary only when the fence line is unindented (`raw` has no leading whitespace); keep `in_fence` tracking unchanged for indented fences.

## 2. Regression coverage

- [x] 2.1 Add `split_regions` unit tests asserting no boundary is inserted before: a 3-space-indented continuation after an ordered item, a 2-space-indented continuation after a bullet, an indented `>` quote line, and an indented code fence — each following a blank line.
- [x] 2.2 Add incremental-equivalence tests that seed a document, apply single-character edits (`replace_range`) at several offsets across the four corrupting fixtures from the proposal, re-derive visual/preview blocks, and assert the derivation counters report zero new full fallbacks (oracle repair) after each edit.
- [x] 2.3 Keep one direct comparison test: incremental blocks equal `MarkdownDocument::from_text(same_text)` blocks for each fixture after an edit.

## 3. Verification

- [x] 3.1 Run `cargo fmt --check` and `cargo test --workspace` (note: pre-existing clippy failures are unrelated; app tests live in the bin target).
- [x] 3.2 Re-run the release-mode diff harness (incremental vs. full across all offsets of the fixtures) and confirm zero mismatches.
- [x] 3.3 `openspec validate fix-incremental-region-list-continuations --strict`.
