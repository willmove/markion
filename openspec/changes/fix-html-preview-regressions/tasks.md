## 1. Raw HTML Preview Blocks

- [ ] 1.1 Update the existing offset-aware preview derivation pass to recognize HTML-only parser paragraphs and emit one source-faithful `PreviewBlock::Html` without changing per-document-version cache behavior.
- [ ] 1.2 Add regression coverage for standalone README-style raw HTML followed by Markdown, plus mixed prose with inline HTML that must remain a rich-text paragraph.

## 2. HTML Text Normalization

- [ ] 2.1 Carry collapsible whitespace across `HtmlPreviewBuilder` text-node and inline-tag boundaries while retaining the originating style and link span metadata.
- [ ] 2.2 Extend HTML preview-part tests to cover separators around styled/linked elements and ensure block boundaries, images, and explicit line breaks retain their existing normalization.

## 3. Verification

- [ ] 3.1 Run the two targeted HTML preview regression tests and `cargo test --workspace`, then resolve any regressions within this change's scope.
- [ ] 3.2 Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `openspec validate fix-html-preview-regressions`.
