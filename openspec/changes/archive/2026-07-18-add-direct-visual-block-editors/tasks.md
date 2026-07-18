## 1. Exact Block Metadata and Edit Protocol

- [x] 1.1 Add typed `VisualBlockEditor` metadata and UTF-8 source subranges for ordinary code, block math, inline images, and table cells
- [x] 1.2 Add pure parsers that accept only byte-exact supported forms and retain the existing source-island fallback for malformed, multiline, reference, unclosed, HTML, front-matter, and diagram constructs
- [x] 1.3 Add a version/id/range-validated `VisualBlockEdit` protocol with one canonical replacement and exact post-edit selection
- [x] 1.4 Add exhaustive range, escaping, stale-event, round-trip, UTF-8, and source-island fallback tests for every editor metadata variant

## 2. Direct Fenced-Code Editor

- [x] 2.1 Render ordinary fenced-code payloads through the shared editable text element without exposing the opening/info/closing fence source
- [x] 2.2 Apply existing memoized syntax highlights to the editable payload and preserve authored indentation, blank lines, info spacing, fence length, and final-newline semantics
- [x] 2.3 Integrate pointer selection, keyboard navigation, platform input, IME geometry, semantic undo, and source-island fallback for unclosed fences
- [x] 2.4 Keep registered diagram fences on their existing complete source-backed editor and add ordinary/unknown-language/diagram regressions

## 3. Direct Block-Math Editor

- [x] 3.1 Present the rendered formula and a compact monospaced LaTeX payload editor together in Visual Edit
- [x] 3.2 Keep the payload editor available through pending, invalid, and failed render states while preserving delimiter bytes
- [x] 3.3 Integrate validation feedback, platform input, CJK/emoji IME, exact candidate geometry, and one-action undo/redo
- [x] 3.4 Add rendered and pure regressions for valid, invalid, fenced, display-dollar, cached, and UTF-8 math editing

## 4. Direct Markdown Image Editor

- [x] 4.1 Present exactly ranged inline images with their preview and direct alt, destination, and optional-title fields
- [x] 4.2 Implement field-specific escaping and one-range replacements without rewriting unrelated delimiters or source bytes
- [x] 4.3 Keep proven fields editable for unavailable local/remote images and preserve request-cache/document-version isolation
- [x] 4.4 Add pointer, Tab traversal, platform input, IME, undo, local/remote/broken image, and ambiguous-reference fallback tests

## 5. Direct GFM Table Cell Editor

- [x] 5.1 Extend the pure table parser with exact authored cell ranges, escaped-pipe handling, and deterministic cell replacement results
- [x] 5.2 Render header and body cells with the shared editable text element while preserving the existing row/column toolbar
- [x] 5.3 Implement Tab/Shift-Tab logical-cell traversal, boundary handoff, active-cell selection restoration, and IME/undo behavior
- [x] 5.4 Preserve alignment markers and exporter semantics across direct cell edits and retain full source fallback for ambiguous tables
- [x] 5.5 Add pure and rendered tests for UTF-8 cells, width reflow, escaped delimiters, row/column operations, alignment, traversal, and one-edit history

## 6. Interaction, Virtualization, and Cache Contracts

- [x] 6.1 Key direct-field focus and pending traversal by document version plus `VisualBlockId`, and reject stale widget events after split/merge/reparse
- [x] 6.2 Preserve visual list scroll anchoring, focused-row mounting, shared preview/visual `Arc` identity, memoized highlighting/math/image state, and debounced Split/Read behavior
- [x] 6.3 Add multi-tab, mode-switch, autosave/recovery, cache-free clone, large-document early-edit, cross-block navigation, and source-mode round-trip regressions
- [x] 6.4 Verify direct-widget focus/hover/layout changes never mutate the document version or invalidate derived Markdown caches

## 7. Verification

- [x] 7.1 Run `cargo fmt --all -- --check`, focused direct-widget/Visual Edit tests, and `cargo test`
- [x] 7.2 Run `cargo test --workspace` and resolve all member-crate, export, doctest, and ignored-test expectation regressions
- [x] 7.3 Run `openspec validate add-direct-visual-block-editors` and resolve all validation errors
