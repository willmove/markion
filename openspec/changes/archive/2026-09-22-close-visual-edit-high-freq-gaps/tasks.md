## 1. YAML collapsible header

- [x] 1.1 Add `VisualBlockKind::FrontMatter` and `VisualBlockEditor::FrontMatter` (payload over the complete authored `---` … `---`/`...` span) and stop emitting a FrontMatter source island in `build_visual_blocks`
- [x] 1.2 Render a collapsible YAML header in Visual Edit (localized label + parsed title when present); invalid YAML still uses the payload editor; drop FrontMatter from `always_source`
- [x] 1.3 Verify pure and GPUI tests: valid/invalid YAML is not an island, title appears when parsed, expand/hover does not bump document version

## 2. Reference-style block images

- [x] 2.1 Prove single-line `![alt][label]`, `![alt][]`, and shortcut `![alt]` spans in `visual_block_editor` / `inline_edit` without accepting multiline or unclosed forms
- [x] 2.2 Keep width/alignment controls gated on `inline_image_at` (inline destinations only) so reference spans are not rewritten to `![alt](url)`
- [x] 2.3 Verify derivation tests that reference images carry an Image editor and no Image island, and that multiline images still refuse the editor

## 3. Ragged GFM tables

- [x] 3.1 Make `table_cell_source_ranges` pad short rows with empty end-of-line ranges and keep extra cells, matching normalized column count
- [x] 3.2 Attach `VisualBlockEditor::Table` whenever those ranges cover the preview grid, so focused ragged tables stay a grid
- [x] 3.3 Verify table unit tests for short and extra-cell rows, plus a Visual Edit test that focus does not island a ragged table

## 4. Math Pending/Error

- [x] 4.1 Always attach `VisualBlockEditor::Math` for display/fenced math (fallback payload = complete authored span when delimiter split fails)
- [x] 4.2 Route MathBlock Pending/Error through `visual_math_editor` and remove the source-island fallback
- [x] 4.3 Verify a GPUI or derivation test that Error/Pending math is not an island and still exposes the payload field

## 5. Coverage matrix, i18n, verification

- [x] 5.1 Localize the YAML header label in all seven languages with exhaustiveness coverage
- [x] 5.2 Update `docs/visual-editing-quality.md` matrix and roadmap: close gaps 1, 4 (reference-style), 5, and 11; leave indented/unclosed code, multiline images, and definition lists open
- [x] 5.3 Run `cargo fmt --check`, `cargo test --workspace` (fix regressions in pairing/YAML/image/table/math/i18n only), and `openspec validate close-visual-edit-high-freq-gaps`
