## 1. Mixed-fragment line grouping

- [x] 1.1 In `visual_text_with_math_element` (`src/app/preview.rs`), group mixed children into logical lines split on projected `\n` / `\r\n` (covering both whitespace-split fragments and unsplittable HardBreak segments); stack those lines as full-width flex-wrap rows instead of one wrapping flex container
- [x] 1.2 Keep navigation icons, math atoms, and HTML image atoms on the line that owns their source end; do not emit a `VisualEditableText` child for the break itself
- [x] 1.3 Add `debug_selector` `visual-mixed-line-{block_index}-{line_index}` on each logical-line row

## 2. Tests and docs

- [x] 2.1 gpui test: the reported fixture (heading + three consecutive lines with a link, email, and inline code) paints at least three mixed-line rows whose tops increase; a wide window so width-wrap cannot fake the break; projection still contains the `\n` bytes
- [x] 2.2 gpui or layout test: a single-line linked paragraph still uses one mixed-line row
- [x] 2.3 Update the Visual Edit support matrix in `docs/visual-editing-quality.md` so mixed fragment layout (links / math / HTML images) is described as preserving authored line breaks

## 3. Verification

- [x] 3.1 `cargo test --bin markion visual_edit` — 28 passed, 0 failed; first compile of the change was warning-free
- [x] 3.2 `openspec validate fix-visual-edit-mixed-prose-line-breaks` passes
