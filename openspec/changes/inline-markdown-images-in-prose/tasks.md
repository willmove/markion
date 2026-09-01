## 1. Preview stream keeps mixed Markdown images inline

- [x] 1.1 Add an image payload on `InlineSpan` and preserve those spans through `append_span` / `finish_rich_text`
- [x] 1.2 At `End(Image)`, attach the image to the open heading, list item, paragraph, quote, or table cell; emit `PreviewBlock::Image` only when no prose container is open or when a flushed paragraph is image-only
- [x] 1.3 Collect inline-span image URLs from preview/visual caches (Read preload, retained refs, remote export scan)

## 2. Visual Edit and Read / Preview presentation

- [x] 2.1 Map `Tag::Image` in `inline_runs` to an `html_image` atom with progressive reveal of the authored `![…](…)` bytes
- [x] 2.2 Paint `InlineSpan.image` in Read / Split Preview mixed layout on the same flex line as adjacent text (compact atom, not full-width block image)

## 3. Tests and docs

- [x] 3.1 Preview + Visual tests for `![alt](url)trailing`, `text ![alt](url) more`, heading/quote, list item (no second bullet), image-only unchanged
- [x] 3.2 Update the Inline Markdown image row in `docs/visual-editing-quality.md`
- [x] 3.3 `cargo test` for the new and existing image/preview tests; `openspec validate inline-markdown-images-in-prose`
