# Tasks: fix-list-item-block-roundtrip

## 1. Renderer fix

- [x] 1.1 In `render_list` (`crates/markdown/src/renderer.rs`), when a `ListItem` has non-empty `blocks`, emit one blank line after the item's inline-content line before the first child block.
- [x] 1.2 Render sibling child blocks with blank-line separation (reuse or mirror `render_blocks` separation at the continuation indent) so consecutive child paragraphs/blocks cannot merge.
- [x] 1.3 Keep tight items (no child blocks) and nested sub-item rendering byte-identical to today.

## 2. Tests

- [x] 2.1 Add renderer unit tests: loose two-paragraph list item round-trips with the child paragraph intact; item with paragraph + fenced-code children keeps both blocks; tight item rendering unchanged; nested sub-list rendering unchanged.
- [x] 2.2 Add a regression test for the diagnosed real-world case: parsing `"- first paragraph\n\n  second paragraph\n"`, rendering, and re-parsing preserves the item's block structure.
- [x] 2.3 Append seed `8b1f2303c3c82c49e7116a1934c95f3ba3c1d2da4ed68a30e09ec56211df7975` to `crates/markdown/tests/roundtrip_property_test.proptest-regressions` (after the fix, so it replays as a passing case).

## 3. Verification

- [x] 3.1 `cargo test -p markdown` passes from the repository root, including `roundtrip_property_test`.
- [x] 3.2 `cargo test --workspace` passes.
- [x] 3.3 `openspec validate fix-list-item-block-roundtrip` passes; archive the change via the OpenSpec workflow after merge.
