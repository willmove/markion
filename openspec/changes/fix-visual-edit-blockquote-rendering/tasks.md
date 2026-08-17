## 1. Model and parse plumbing for the alert kind

- [x] 1.1 Add `AlertKind` (Note/Tip/Important/Warning/Caution) to `src/model.rs` and an `alert: Option<AlertKind>` field on `PreviewBlock::BlockQuote`; update the single construction site in `src/lib.rs`
- [x] 1.2 Capture the kind from `Tag::BlockQuote(kind)` / `TagEnd::BlockQuote(kind)` at quote depth 1 in `src/lib.rs` and map pulldown-cmark's kind onto `AlertKind`; deeper quotes stay uncaptured
- [x] 1.3 Keep body-less alert quotes in `push_nonempty_block` (`src/parse.rs`) when `children.is_empty() && alert.is_some()`; add parse-level tests (alert quote, plain quote, lone `> [!NOTE]`)

## 2. Soft-break synthesis for quoted leaves

- [x] 2.1 In `visual_block_from_preview` (`src/visual.rs`), after `inline_runs` and only when the row has a quote context, insert a synthetic `"\n"` run owning exactly the first newline byte of each unowned gap that contains one; keep runs sorted by source range
- [x] 2.2 Unit tests: multi-line quoted paragraph renders with a line break and keeps a monotonic display↔source mapping; `> [!CUSTOM]` body renders on its own line; hard-break spacing (`> a␣␣` / `> b`) still breaks; unquoted paragraphs produce no synthetic runs; existing quote fixtures (`quoted_mixed_chinese_fixture…`, `quoted_nested_lists…`) stay green

## 3. Callout title row

- [x] 3.1 Add `VisualBlockKind::CalloutTitle { kind: AlertKind }` to `src/model.rs`
- [x] 3.2 In `build_visual_blocks` (`src/visual.rs`), route an alert quote group's leading marker-line gap (and body-less groups) to a `CalloutTitle` block owning exactly those bytes: no editable runs, `quote_context` with marker ranges covering the `> ` prefix and the `[!NOTE]` bytes, edge assignment as group First/Only
- [x] 3.3 Caret/reveal integration: verify the title row is reachable by keyboard navigation and click via the marker-only-row machinery and reveals `> [!NOTE]` verbatim when focused; if pointer placement cannot land, apply the design's fallback (one conservative editable run for the `[!NOTE]` bytes); pin the chosen behavior in a test
- [x] 3.4 Unit tests for the reported fixture (`> [!NOTE]` + CJK body with inline link): no `Unsupported` island, contiguous byte coverage with no overlap, title row plus body rows in one quote group; body-less alert; pin pulldown behavior for `> [!NOTE] trailing`; unknown marker stays literal; nested-quote behavior unchanged

## 4. View layer

- [x] 4.1 Add the `CalloutTitle` arm in `visual_block_view` (`src/app/preview.rs`): bold canonical label (Note/Tip/Important/Warning/Caution) with per-kind accent color from the app palette, degrading to the existing quote gray; keep quote-group decoration
- [x] 4.2 Update remaining exhaustive matches for the new block kind (`src/document_memory.rs`, `src/source_mapped.rs`, `src/block_edit.rs`, `src/app/editing.rs`): title row is a structural quote-group leaf — not reorderable, no transform chrome — following existing quote-leaf exclusions

## 5. Verification

- [x] 5.1 `cargo test` (root) and `cargo test --workspace` pass; build stays warning-free under `-D warnings`
- [x] 5.2 Manual check in the app: the fixture renders as one callout across light/dark themes; existing blockquote documents render unchanged; Enter/Backspace around the marker line do not corrupt source
- [x] 5.3 `openspec validate fix-visual-edit-blockquote-rendering` passes, then archive via `/openspec:archive`
