# Tasks: fix-visual-edit-inline-markdown-images

## 1. Failing regression tests

- [x] 1.1 Add a visual test in `src/visual.rs` for the reported fixture (`**已订阅Google AI Pro**\n![image.png](https://example.com/a.png)` with no blank line). Assert two disjoint rows: `Paragraph` then `Image`; the image is not `Unsupported`; the paragraph's visible runs do not contain the alt text `image.png`; every source byte is owned by exactly one row
- [x] 1.2 Add visual tests for `hello ![alt](url) world` (three rows: Paragraph, Image, Paragraph) and for two images with prose between them (alternating prose/image in source order, disjoint ranges)
- [x] 1.3 Add characterization tests that image-only paragraphs and blank-line-separated `Intro\n\n![alt](url)` stay one `Image` row (plus the leading paragraph when present) with no new partition artifacts
- [x] 1.4 Add a quoted-paragraph fixture (`> text\n> ![alt](url)`) asserting both partitioned rows carry quote context and the image is not an `Unsupported` island

## 2. Visual-layer partition (design D1)

- [x] 2.1 In `build_visual_blocks` (`src/visual.rs`), after quote expansion and the nested-list truncate, partition `Paragraph`/`Heading` leaves whose source range contains following `PreviewBlock::Image` leaves: emit nonempty prose slices, the image leaves, and trailing/interstitial continuations that reuse the parent `PreviewBlock` pointer, kind, and `quote_group`, then skip empty (`start >= end`) slices
- [x] 2.2 Run the tests from 1.x and fix off-by-one slice bounds (trailing newline stays on the preceding prose slice; image ranges remain the pulldown image tag range)
- [x] 2.3 Pin that a continuation row's transform/duplicate/delete source unit is the slice, not the original full paragraph, so deleting leftover prose cannot swallow the image row

## 3. Documentation and verification

- [x] 3.1 Update the Inline Markdown image row in `docs/visual-editing-quality.md` to describe mixed-paragraph partition (prose row + image row + leftover prose) and name list-item inline images as still conservative
- [x] 3.2 Run `cargo test --workspace` and resolve failures in this change's scope
- [x] 3.3 Run `openspec validate fix-visual-edit-inline-markdown-images`
