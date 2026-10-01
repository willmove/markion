## 1. Regression evidence

- [x] 1.1 Add a failing Visual Edit unit test for the Notes fixture covering footnote superscript runs, single footnote-definition block, and non-island link reference definition coverage
- [x] 1.2 Confirm the test fails for the current symptoms before applying production fixes

## 2. Footnote reference resolution

- [x] 2.1 Collect document footnote definition stubs (skipping fenced code) alongside link reference definitions
- [x] 2.2 Append footnote stubs in `inline_runs` so `FootnoteReference` events resolve without shifting in-block source ranges

## 3. Footnote definition blocks

- [x] 3.1 Add `PreviewBlock::FootnoteDefinition` and track footnote depth in `derive_preview_and_outline` so definition bodies are not ordinary paragraphs
- [x] 3.2 Map footnote definitions to a Visual Edit block kind without Unsupported source-island chrome and render them in the visual surface
- [x] 3.3 Update source-range / match arms that exhaust `PreviewBlock` / `VisualBlockKind`

## 4. Link reference definition gaps

- [x] 4.1 Detect gaps whose non-blank lines are only link reference definitions and emit `ReferenceDefinition` blocks without source-island chrome
- [x] 4.2 Render reference-definition rows as editable source text without the Unsupported island box

## 5. Verification

- [x] 5.1 Make the Notes regression test pass and run related `visual::tests` / preview tests
- [x] 5.2 `openspec validate fix-visual-edit-footnotes-and-link-defs`
