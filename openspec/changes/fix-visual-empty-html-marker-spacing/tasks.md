## 1. Compact marker row

- [x] 1.1 Route content-free HTML-only paragraphs to the HTML block renderer: `html_only_paragraph_source` also accepts tag runs with no visible text at any depth. Verify with a lib unit test covering anchors, empty style tags, and whitespace-only containers versus text-bearing tags, plus the document-level `Html` block ownership for the fixture.
- [x] 1.2 Refactor `html_preview_block_view` to accept precomputed parts (`html_preview_block_view_with_parts`) and add an `html_parts_have_visible_content` helper (Image/Table always visible; Text visible only with non-whitespace rich text). Verify with a parse-level test that the anchor and whitespace-only containers flatten to no visible parts while `<div>text</div>`, images, and tables stay visible.
- [x] 1.3 In `visual_html_editor`, render content-free HTML blocks as one slim muted monospace line containing the payload field element (field range excluding the trailing separator) — no border, background, padding, or block margins. Verify with a GPUI regression asserting the anchor row spans about one paragraph line and still paints a caret and accepts typed input with exact source round-trip.

## 2. Integration verification

- [x] 2.1 Run `cargo test` for the root package, `cargo fmt`, and `openspec validate fix-visual-empty-html-marker-spacing`.
