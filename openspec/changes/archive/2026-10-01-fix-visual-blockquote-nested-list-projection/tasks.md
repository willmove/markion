## 1. Ordered Blockquote Model

- [x] 1.1 Replace the split `BlockQuote { text, children }` representation with one authored-order child flow and update `derive_preview_and_outline` so quoted paragraphs and list items are appended at their semantic end events.
- [x] 1.2 Update `PreviewBlock::plain_text`, source-range access/shifting, preview selection runs, and Split Preview / Read rendering to fold and render ordered blockquote children without paragraph/list reordering.
- [x] 1.3 Audit statistics, LaTeX/DOCX export, math collection, search/text extraction, and retained-memory accounting so ordered quoted children are included exactly once.
- [x] 1.4 Add parser and consumer tests for paragraph-only quotes, intro/list/outro order, ordered start indices, unordered/task/nested lists, inline formatting/math, and exact plain-text/export order.

## 2. Non-Overlapping Visual Projection

- [x] 2.1 Add GPUI-independent quote-context metadata to `VisualBlock` for depth, exact quote-marker ranges, and contiguous-group edge position; update shifting, equality, and retained-size accounting.
- [x] 2.2 Replace parent-plus-child blockquote expansion in `build_visual_blocks` with an ordered leaf flattener and source partitioner that assigns every supported quote byte to one monotonic, contiguous, UTF-8-safe visual row.
- [x] 2.3 Classify structural-only `>` lines and blank quoted separators as quote-context whitespace or adjacent owned source so they never create an unexpected non-whitespace `Unsupported` gap.
- [x] 2.4 Preserve the existing nested-list subtree partitioning inside quotes and keep the overlap guard as a failing correctness backstop rather than bypassing it.
- [x] 2.5 Add pure model tests using the reported Chinese fixture and minimal variants to assert no duplicate projected text, no overlap, complete source coverage, correct quote context, and no `Unsupported` rows.

## 3. Composite Prefixes and Source-Faithful Punctuation

- [x] 3.1 Extend prefix derivation to identify repeated quote markers separately from an optional ordered, unordered, or task-list leaf prefix, including indentation and exact UTF-8 source ranges.
- [x] 3.2 Update projection marker hiding/reveal and source/display boundary mapping so quote and leaf prefix layers remain monotonic and only the active exact layer is revealed.
- [x] 3.3 Implement quoted structural transitions: Enter continues the combined quote/list prefix with correct numbering/check state, and Backspace demotes the inner list prefix before the quote prefix, each as one undoable source mutation.
- [x] 3.4 Add a Visual Edit parser-option helper that preserves all semantic extensions except smart punctuation, leaving Preview / Read / export parser options unchanged.
- [x] 3.5 Add tests for ASCII single/double quotes and dash sequences in quoted paragraphs/list items, proving they stay rendered, source-exact, editable, and do not trigger a complete source island.

## 4. Quoted Row Rendering and Interaction

- [x] 4.1 Decorate the existing paragraph/list/task row renderers with quote depth, border, padding, typography, and first/middle/last spacing instead of introducing a recursive multi-editor GPUI row.
- [x] 4.2 Verify ordered markers, bullets, task states, and nested indentation render inside a visually continuous quote boundary without duplicated quote/list marker chrome.
- [x] 4.3 Add rendered-window tests covering pointer placement, keyboard navigation across quoted siblings, selection/copy, CJK/emoji input, IME marked-range geometry, prefix reveal, and undo/redo.
- [x] 4.4 Manually verify the reported Markdown fixture in Visual Edit and confirm the same document remains correct in Edit, Split Preview, and Read modes.

## 5. Incremental and Stable-Identity Guarantees

- [x] 5.1 Extend visual-range shifting and stable-ID reconciliation for quote-context metadata, proving unchanged quoted siblings retain identity after an earlier local edit while changed/split/merged rows receive new IDs.
- [x] 5.2 Add incremental-versus-fresh-full tests for UTF-8 edits, CRLF, blank quoted separators, list continuations, prefix insertion/removal, and quote/list block splits or merges without debug-oracle fallbacks.
- [x] 5.3 Add a cache regression proving caret, selection, hover, scroll, and repaint changes reuse the same per-version `Arc<VisualBlock>` model and perform no render-frame Markdown parse.

## 6. Documentation and Validation

- [x] 6.1 Update `docs/visual-editing-quality.md` to classify mixed blockquote/list flows as rendered source-backed rows and document source-faithful Visual Edit punctuation versus Preview smart punctuation.
- [x] 6.2 Run focused blockquote, visual projection, interaction, export, and source-mapped tests; then run `cargo test --workspace` and fix all regressions without weakening source-range assertions.
- [x] 6.3 Run `cargo fmt --check`, `openspec validate fix-visual-blockquote-nested-list-projection`, and the repository quality gate required by `docs/visual-editing-quality.md`.
