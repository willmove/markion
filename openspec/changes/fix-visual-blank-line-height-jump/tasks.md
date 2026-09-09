## 1. Regression feedback loop

- [x] 1.1 Add a rendered GPUI regression that clicks an authored blank line, types a one-line paragraph, and asserts the replacement row height and following-row position do not change.
- [x] 1.2 Run the focused regression before the fix and record that it fails on the reported geometry mismatch.

## 2. Stable whitespace geometry

- [x] 2.1 Keep whitespace content/caret/click line geometry line-height-based and assign paragraph spacing once to the end of each same-context paragraph/whitespace body-flow group.
- [x] 2.2 Cover margin ownership, default/customized paragraph spacing, paragraph merging, multi-line whitespace, and pathological-bound source/caret geometry.

## 3. Documentation and verification

- [x] 3.1 Update the Visual Edit quality matrix to describe stable body-flow paragraph spacing for authored blank lines.
- [x] 3.2 Run focused whitespace/Visual Edit tests, `cargo test --workspace`, and `openspec validate fix-visual-blank-line-height-jump`.
