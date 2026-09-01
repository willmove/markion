## 1. Partition discovers images that precede the parent

- [x] 1.1 Update `partition_prose_around_nested_images` in `src/visual.rs` so an `Image` leaf contained in a later paragraph/heading parent is not emitted early; the parent collects every contained image (before or after in the leaf list), sorted by source start, and emits the existing disjoint slices (empty leading prose skipped)
- [x] 1.2 Keep list-item images unpartitioned (no later paragraph/heading parent owns them; overlap guard remains the fallback)

## 2. Tests

- [x] 2.1 Add a Visual Edit derivation test for `![alt](url)trailing text`: image row then continuation paragraph, complete disjoint coverage, no `Unsupported` island, continuation runs do not contain `![` or the destination URL
- [x] 2.2 Add a quoted-paragraph / heading variant of the leading-image fixture (quote context preserved; syntax not duplicated)
- [x] 2.3 Keep existing mixed-paragraph tests green: `text ![alt](url) more`, multiple images, image-only, blank-line-separated

## 3. Docs and validation

- [x] 3.1 Name the leading-image partition in the inline Markdown image row of `docs/visual-editing-quality.md`
- [x] 3.2 `cargo test` for the new and existing visual image-partition tests
- [x] 3.3 `openspec validate fix-visual-leading-markdown-image`
