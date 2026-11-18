## 1. Reproduction

- [x] 1.1 Reproduce both reports on main before changing the implementation.

On Windows at `6e3fdcc`, the #30 right body cell mapped to `MAPLE_46`, direct editing changed that value, and Format added a third column. The #31 table lookup returned `0..61` and Format turned `Notes | choices` into the header, although the parser emitted it as a separate paragraph.

## 2. Implementation

- [x] 2.1 Align GFM cell splitting and caret column lookup. Verify direct edits, Format, and authored backslash runs in the rendered table.
- [x] 2.2 Use parser-owned table ranges for lookup and table commands. Verify adjacent prose and header ownership, source bytes, line endings, and unaffected tables.
- [x] 2.3 Verify the editor command paths, including Undo and structural validation, through the existing test harness.

## 3. Verification

- [x] 3.1 Run the regression tests, locked workspace tests, formatting check, and strict OpenSpec validation.

The six new Windows regression tests passed, including Visual Edit click and input, the Source Format Table command, and byte-exact Undo. The locked workspace run finished with 2,253 passed, zero failed and five ignored. The formatting check and strict OpenSpec validation passed.
