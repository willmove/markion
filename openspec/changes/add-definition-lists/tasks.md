## 1. Parsing and model

- [x] 1.1 Enable `ENABLE_DEFINITION_LIST`; add `PreviewBlock::DefinitionTerm` / `DefinitionDetail`, `VisualBlockKind` counterparts, `VisualBlockPrefixKind::DefinitionMarker`, and build terms and details in the preview event loop (tight, loose, nested-block cut, inside quotes); verify with `cargo check --bins`
- [x] 1.2 Add lib tests for preview blocks (tight, multiple, loose, nested list, quote) and Visual Edit rows (kinds, `:` prefix, runs, single ownership); verify with `cargo test --lib definition_list`

## 2. Rendering

- [x] 2.1 Render terms bold and details indented in Split Preview / Read and in Visual Edit (marker hidden through the prefix); verify with `cargo check --bins`
- [x] 2.2 Add app tests: Read mode shows term and detail rows; Visual Edit shows them and typing in a detail edits its source exactly once; verify with `cargo test --bin markion definition_list`

## 3. Export

- [x] 3.1 HTML `<dl>` via the parser option; bold terms and following details in DOCX (indented), PDF, and LaTeX (`quote`); verify with `cargo check --bins`
- [x] 3.2 Add export tests for HTML, LaTeX, DOCX, and PDF definition-list output; verify with `cargo test --lib definition_list_export`

## 4. Documentation and integration

- [x] 4.1 Update the coverage matrix and roadmap in `docs/visual-editing-quality.md`
- [x] 4.2 Run `cargo fmt --check`, `cargo test --bin markion`, `cargo test --lib`, and confirm no new clippy lints on changed lines; validate with `openspec validate add-definition-lists --strict`
