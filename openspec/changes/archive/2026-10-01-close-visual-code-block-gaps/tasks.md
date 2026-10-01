## 1. Model and parsing

- [x] 1.1 Add `fenced: bool` to `PreviewBlock::CodeBlock` from `CodeBlockKind` and `VisualEditorFieldKind::IndentedCodePayload`; verify with `cargo build` and `cargo test --lib`

## 2. Visual Edit editors

- [x] 2.1 Give indented code an `IndentedCodePayload` `Code` editor and unclosed or malformed fences a payload running to the block end (`fenced_payload_ranges(.., allow_unclosed)`); update the old island-expecting tests and add `unclosed_fence_keeps_a_payload_editor_to_the_block_end` and `indented_code_gets_a_dedented_payload_editor`; verify with `cargo test --lib visual`
- [x] 2.2 Dedented projection, highlighting, and copy; no language chip; indentation-preserving sanitizer for Enter and paste; auto-pair restriction; dedented `block_body` for transforms; add `indented_code_transforms_with_its_indentation_removed` and the app test `visual_indented_code_enter_keeps_lines_in_the_block`; verify with `cargo test --lib indented_code` and `cargo test --bin markion visual_indented_code`

## 3. Documentation and integration

- [x] 3.1 Update the coverage matrix and roadmap in `docs/visual-editing-quality.md` (indented code and unclosed fences closed); verify by reading the roadmap table
- [x] 3.2 Run `cargo fmt --check`, `cargo test --bin markion`, `cargo test --lib`, and confirm no new clippy lints on changed lines; validate with `openspec validate close-visual-code-block-gaps --strict`
