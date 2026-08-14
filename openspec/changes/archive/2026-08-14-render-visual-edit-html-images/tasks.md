## 1. Exact `<img>` tag recognizer (shared seam)

- [x] 1.1 Expose a narrow helper from `src/parse.rs` that parses a source slice as exactly one complete non-closing `<img …>` tag (self-closing or void) with a non-empty `src`, returning alt/url/title; unit tests for self-closing, unquoted/quoted attrs, missing `src`, closing tag, and multi-tag slices.

## 2. Model + inline derivation

- [x] 2.1 Add `VisualHtmlImage` payload to `VisualInlineRun` and `VisualRevealKind::HtmlImage` in `src/model.rs`, updating all run construction sites.
- [x] 2.2 In `inline_runs`, classify each inline-HTML event: image-only tags emit image runs (byte-exact `visible_text`, `source_range == content_range`) plus reveal candidates; any other inline HTML sets the block-level HTML flag so the whole-block source island remains. Table/paragraph/list/quote/heading/footnote derivation tests cover no-island vs island cases.
- [x] 2.3 Extend `reveal_candidate_is_exact` for `HtmlImage` (starts with `<img`, ends with `>`) and include the kind in the projection's caret-end `include_end` rule next to math; projection tests assert reveal-on-focus and restore-on-blur text.

## 3. Rendering + image lifecycle

- [x] 3.1 Add the inline image atom to the mixed text/math element path (selection highlight, start/end hit targets placing the caret at tag boundaries, `preview_image_view` content) and thread `document_dir` from `visual_block_view`.
- [x] 3.2 Extend `collect_preview_image_urls` to claim inline image run URLs; claim/eviction test mirroring the block-level image one.

## 4. Contracts, quality gate, archive

- [x] 4.1 Update `docs/visual-editing-quality.md` support matrix: split the HTML row (rendered HTML blocks; inline `<img>` rendered with progressive reveal; other inline HTML island) and note table-cell flattening.
- [x] 4.2 Run `pwsh ./scripts/check-quality.ps1` (fmt, `cargo test --workspace`, strict OpenSpec validation) and resolve findings within scope.
- [x] 4.3 `openspec validate render-visual-edit-html-images --strict`, then archive the change.
