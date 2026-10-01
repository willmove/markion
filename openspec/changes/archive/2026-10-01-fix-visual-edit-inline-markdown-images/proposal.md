# Proposal: fix-visual-edit-inline-markdown-images

## Why

In Visual Edit, a CommonMark paragraph that mixes prose with a Markdown image — including the common “bold line, then `![alt](url)` on the next line with no blank line” pattern — does not show the image. The alt text leaks as ordinary copy and the image syntax appears in a gray source island, while Read mode renders the same source correctly. The shared preview stream emits both a parent `Paragraph`/`Heading` whose source range still covers the image and a nested `PreviewBlock::Image`; Visual Edit’s overlap guard then force-marks the image `Unsupported`, and `always_source` always draws the island.

## What Changes

- Visual Edit SHALL partition a paragraph or heading whose source range swallows one or more nested Markdown `PreviewBlock::Image` ranges, so each visual row owns a disjoint slice: prose before the image, the image as `VisualBlockKind::Image`, and leftover prose after the image as a continuation row.
- Nested image rows SHALL keep the existing image presentation (bounded preview, caption, missing-resource placeholder) and SHALL NOT be force-marked `Unsupported` solely because they started life inside a prose block.
- Prose slices SHALL re-parse only their owned source range, so alt text no longer leaks into the parent row and the image syntax is not duplicated on screen.
- Regression tests SHALL pin the reported no-blank-line fixture, same-line `text ![img](url) more`, multiple images in one paragraph, image-only paragraphs (unchanged), and blank-line-separated images (unchanged).
- The Visual Edit support matrix SHALL name this partition as the presentation for mixed-paragraph Markdown images, with conservative fallback only for unproven/ambiguous image syntax already covered by the image-island rules.

### Non-goals

- No inline-atom rendering of Markdown `![...](...)` inside a single prose run (the HTML `<img>` path); same-line images become stacked rows (prose, image, prose), not baseline-aligned atoms.
- No change to list-item + inline Markdown image (splitting would emit a second bullet); that remains the pre-existing overlap quirk called out by `fix-visual-edit-list-nested-code`.
- No change to Read / Split Preview / export block streams, canonical source text, image loading, or focused-caret source-island behavior for standalone image-only blocks.
- No new image field editors (alt/destination/title stay as they are today).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: extend *Source-backed Visual Edit mode* so a paragraph or heading that contains a nested Markdown image renders disjoint visual rows (prose + `VisualBlockKind::Image` + leftover prose) instead of overlapping into an `Unsupported` source island.

## Impact

- **Code**: `src/visual.rs` (`build_visual_blocks` partition of prose leaves around nested `PreviewBlock::Image`); tests in `src/visual.rs` and possibly `src/lib.rs`. View-layer `VisualBlockKind::Image` rendering in `src/app/preview.rs` is reused, not redesigned.
- **Docs**: `docs/visual-editing-quality.md` support-matrix row for inline Markdown images.
- **Invariants**: derived Visual Edit blocks remain cached per document version and shared via `Arc`; partition runs during that derivation, not on caret movement or keystroke. Incremental source-mapped output must still equal a full parse of the same text. No `gpui` dependency in `crates/*`.
- **Compatibility**: pure Visual Edit derivation/presentation fix — no file format, settings, or API migration.
