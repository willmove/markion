# Tasks: fix-visual-edit-list-nested-code

## 1. Failing regression tests

- [x] 1.1 Add a parser test (src/lib.rs tests) deriving preview blocks for a list item whose indented continuation holds a fenced code block; assert the stream order is ListItem → CodeBlock → ListItem → CodeBlock and that each item's source range ends no later than its nested code block's start
- [x] 1.2 Add a visual test (src/visual.rs tests) running `build_visual_blocks` over the same pattern; assert the item rows are `ListItem` kind with no `Unsupported` island, the nested code rows carry a `VisualBlockEditor::Code`, and no source byte is owned by two rows (no duplicated text)
- [x] 1.3 Add a `fenced_payload_ranges` unit test for a nested fence whose payload and closing lines retain 4-space list indentation, asserting exact opening/payload/info/closing ranges

## 2. Parser ordering fix (design D1)

- [x] 2.1 In `derive_preview_and_outline` (src/lib.rs), stable-sort the finished block stream by `source_range.start` before returning so list-nested blocks restore document order
- [x] 2.2 Run `cargo test` and reconcile any characterization tests that pinned the previous event-order quirk; add fixture coverage for tables, alerts, footnotes, nested lists, and blockquotes to pin their ordering

## 3. Visual partition generalization (design D2)

- [x] 3.1 Partition the swallowed list-item range — implemented in the parse layer (item-range truncation in `derive_preview_and_outline`, see design D2 revision); confirmed `build_visual_blocks` needs no change for this pattern
- [x] 3.2 Verify caret/whitespace handling at the partition boundary (trailing-whitespace run absorbs the item's trailing blank lines; no spurious gap box) with a focused test

## 4. Nested fence editor ranges (design D3)

- [x] 4.1 In `fenced_payload_ranges` (src/visual.rs:1078), keep the strict `indent <= 3` closing-fence scan, then fall back to measuring the common indentation of non-blank payload lines and accepting a closing fence at exactly that indentation with an empty remainder
- [x] 4.2 Add edge-case tests: payload line consisting only of a longer backtick run, blank first payload line, tilde fences nested in lists, and ordered-list indentation widths

## 5. Documentation and verification

- [x] 5.1 Update the Visual Edit support matrix in docs/visual-editing-quality.md: list items with nested fenced code blocks move from conservative-source-island fallback to rendered list row + `VisualBlockEditor::Code`, naming the new fallback triggers
- [x] 5.2 Run `cargo test --workspace`
- [x] 5.3 Manually verify the reported document (大模型服务API和账号信息2026Q3.md, MiniMax section) in Visual Edit (rendered rows, no raw boxes, no duplication) and Read mode (code below its bullet)
- [x] 5.4 Run `openspec validate fix-visual-edit-list-nested-code`
