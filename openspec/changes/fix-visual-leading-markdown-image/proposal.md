## Why

In Visual Edit, a paragraph that **starts** with a Markdown image and then continues with prose on the same line — `![alt](url)trailing text` — renders the image **and** still shows the complete authored `![alt](url)` syntax underneath. Read mode presents the same source as one image plus trailing copy. The existing mixed-paragraph partition (`fix-visual-edit-inline-markdown-images`) only looks *forward* from a prose parent for nested image leaves; pulldown-cmark emits the `Image` block *before* the parent when both share the same start offset, so this leading-image shape never partitions and the overlapping parent is force-marked `Unsupported`.

## What Changes

- Visual Edit SHALL partition a paragraph or heading whose nested Markdown image starts at the same source offset as the parent, producing disjoint rows: the image as `VisualBlockKind::Image`, then leftover trailing prose. The authored `![…](…)` bytes SHALL belong only to the image row.
- The image row SHALL keep the existing image presentation (bounded preview, caption, placeholder) and SHALL NOT also appear as a raw-source island below the preview.
- Trailing prose SHALL re-parse only its owned range, so alt text and destination syntax do not leak into the continuation row.
- Regression tests SHALL pin the reported leading same-line fixture (`![image.png](https://…)和其他…`), plus leading image with no trailing prose (already image-only), image not at offset zero (`text ![img](url) more`, already covered), and multiple leading-adjacent images if the stream can emit them.
- The Visual Edit support-matrix row for inline Markdown images SHALL name this leading-image partition explicitly.

### Non-goals

- No inline-atom rendering of Markdown `![…](…)` inside a single prose run (HTML `<img>` remains the inline path). Leading images still become stacked rows (image, then leftover prose), matching the existing mixed-paragraph contract.
- No change to list-item inline Markdown images (second-bullet problem; overlap fallback stays).
- No change to Read / Split Preview / export block streams, canonical source text, image loading, or HTML-table rendering.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Visual Edit mixed-paragraph Markdown images must partition when the image is the first construct in the parent (same start offset as the paragraph/heading), so the authored image syntax is not duplicated under the preview.

## Impact

- **Code:** `src/visual.rs` (`partition_prose_around_nested_images` and/or the leaf order it assumes); tests in `src/visual.rs`. View-layer `VisualBlockKind::Image` is reused.
- **Docs:** `docs/visual-editing-quality.md` support-matrix row for inline Markdown images.
- **Invariants:** derived Visual Edit blocks remain cached per document version and shared via `Arc`; partition still runs during that derivation, not on caret movement or keystroke. Incremental source-mapped output must still equal a full parse of the same text. No `gpui` dependency in `crates/*`.
- **Compatibility:** presentation-only Visual Edit fix — no file format, settings, or API migration.
