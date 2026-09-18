## 1. Derived image presentation

- [x] 1.1 Carry parsed Markdown image presentation metadata from image-title parsing through `ImageDraft`, `InlineImage`, `PreviewBlock::Image`, and `VisualBlockKind::Image`; verify all constructors and matches compile and existing image parser tests remain green.
- [x] 1.2 Add a parser/model regression test proving `{width=50 align=center}` survives into the standalone preview image block and inline image representation.

## 2. Rendered-mode layout

- [x] 2.1 Correct Visual Edit Markdown image alignment to use horizontal flex justification while preserving resize handles, captions, and intrinsic aspect ratio; verify with the focused Visual Edit image test.
- [x] 2.2 Apply the cached image presentation to Read and Split Preview using a percentage-sized, max-width-bounded wrapper and horizontal justification; verify the sample image is centered and rendered at half the available row width.

## 3. Regression verification

- [x] 3.1 Add one cross-mode GPUI regression test covering Read, Split Preview, and Visual Edit for a centered 50%-width standalone Markdown image.
- [x] 3.2 Run `cargo fmt --all -- --check`, focused image/parser tests, `cargo test --workspace`, `git diff --check`, and `openspec validate fix-markdown-image-preview-presentation`.
