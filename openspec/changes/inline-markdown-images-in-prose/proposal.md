## Why

A Markdown line that places an image and trailing prose in the same paragraph — `![alt](url)trailing text` — is authored as one inline flow (Typora, CommonMark). After `fix-visual-leading-markdown-image`, Visual Edit no longer leaks the `![…](…)` source under the preview, but it still **stacks** the image on its own row and the trailing sentence below. Read / Split Preview do the same because `derive_preview_and_outline` extracts every `Tag::Image` as `PreviewBlock::Image` while the paragraph is still open. That is not the authored layout.

## What Changes

- Mixed Markdown images (image plus any other prose in the same paragraph, heading, quoted paragraph, or list item) SHALL stay in that construct as inline image atoms, on the same visual line as adjacent text, in **Visual Edit and Read / Split Preview**.
- The complete authored `![alt](url)` bytes SHALL belong only to that atom. They SHALL NOT appear as a source island, as leaked alt/destination copy, or as a second stacked image row.
- Image-only paragraphs (and blank-line-separated standalone images) SHALL remain `PreviewBlock::Image` / `VisualBlockKind::Image` with the existing caption, width, and alignment controls.
- Visual Edit mixed images SHALL reuse the existing HTML `<img>` inline-atom path (`html_image` run, progressive reveal of the authored syntax when focused).
- Regression tests SHALL pin the reported `![image.png](url)和其他…` fixture in both the preview block stream and Visual Edit, plus `text ![alt](url) more`, headings/quotes, and list items (no second bullet).

### Non-goals

- No change to standalone image-only field editors (alt/destination/title, replace, width, alignment).
- No HTML-table rendering work (separate thread).
- No new image loader, cache, or remote-fetch semantics.
- Export PDF/DOCX/LaTeX keep mixed images from being dropped, but do not need Typora-identical inline layout in this change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: mixed Markdown `![…](…)` in prose is an inline atom in Visual Edit and Read/Preview, not a stacked image row plus leftover paragraph. This supersedes the stacked-row partition contract in `fix-visual-edit-inline-markdown-images` and `fix-visual-leading-markdown-image`.

## Impact

- **Code:** `src/lib.rs` / `src/parse.rs` (stop extracting mixed images as `PreviewBlock::Image`); `src/model.rs` (`InlineSpan` image payload); `src/visual.rs` (`inline_runs` `Tag::Image`); `src/app/preview.rs` and `src/app/preview_image.rs` (Read mixed layout); image-URL collection; tests; `docs/visual-editing-quality.md`.
- **Invariants:** derived preview and Visual Edit blocks remain cached per document version and shared via `Arc`. Inline atoms are computed during that derivation, not on caret movement. Incremental source-mapped output must still equal a full parse. No `gpui` in `crates/*`.
- **Compatibility:** presentation-only. Canonical Markdown is unchanged.
