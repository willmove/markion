## Context

`inline_edit` already parses `{width=N align=left|center|right}` into `ImagePresentation`, and Visual Edit currently reparses the exact image only to obtain that data. The main preview parser instead stores only URL, title, and identity in `InlineImage` and `PreviewBlock::Image`; Read/Split therefore have no presentation data to apply. Visual Edit wraps the image in a row but currently calls `items_*`, which aligns the cross axis of a row rather than the horizontal main axis.

The preview blocks are derived and cached per document version. The fix must keep presentation data in that derived model instead of adding a render-time source scan.

## Goals / Non-Goals

**Goals:**

- Preserve one parsed `ImagePresentation` value from Markdown parsing to all rendered image surfaces.
- Render standalone Markdown images inside a full-width row with a percentage-sized, max-width-bounded image wrapper and horizontal justification.
- Keep natural image aspect ratio by leaving intrinsic image sizing in the existing image loader/view.
- Cover default and explicit presentation behavior without changing canonical source serialization.

**Non-Goals:**

- Do not change raw HTML image behavior fixed by the adjacent change.
- Do not change the supported metadata grammar or image resizing drag semantics.
- Do not change inline mixed-prose image atoms or export output.

## Decisions

### Preserve presentation in derived image types

Add `Option<ImagePresentation>` to `ImageDraft`, `InlineImage`, `PreviewBlock::Image`, and `VisualBlockKind::Image`. Parse the metadata once when the pulldown-cmark image event is converted into an image draft, then copy it through paragraph extraction and visual-block derivation. Keep it optional so non-Markdown/generated image descriptors and existing constructors can retain their prior semantics; renderers use `ImagePresentation::default()` when absent.

The alternative of calling `inline_image_at` from Read/Split render code was rejected because it reparses document source during rendering and would weaken the per-version derived-state invariant.

### Share a percentage/alignment presentation wrapper

Use a full-width flex row whose main-axis justification maps `Left`/`Center`/`Right` to `justify_start`/`justify_center`/`justify_end`. Its child wrapper receives the clamped percentage width and `max_w_full()`, then contains the existing `preview_image_view`. This keeps pending/error placeholders in the same geometry, preserves the image's intrinsic aspect ratio, and prevents oversized authored percentages from overflowing narrow panes.

Visual Edit retains its existing resize handle and caption behavior, but switches its alignment row to the same horizontal justification semantics.

### Keep metadata parsing shared

Expose a crate-visible helper from `inline_edit` that extracts presentation metadata from an image title. The parser uses that helper for `ImageDraft`; the existing exact-image lookup continues to use the same split/strip logic for source-backed editing and captions.

## Risks / Trade-offs

- [Risk] Adding a field to image model variants requires updating constructors and pattern matches across cache, export, visual, and tests. → Mitigation: keep the field optional and run the full compiler/test suite plus focused image tests.
- [Risk] A percentage-sized wrapper could distort or overflow images if applied to the decoded image itself. → Mitigation: size only the wrapper, keep the existing intrinsic image view inside it, and apply `max_w_full()`.
- [Risk] Changing the default block-level Markdown image alignment could affect snapshots that relied on the old accidental left alignment. → Mitigation: the documented presentation default is centered; add explicit cross-mode geometry coverage and inspect focused existing image tests.

## Migration Plan

No file-format migration is needed. Existing documents without metadata receive `ImagePresentation::default()` at render time; authored metadata continues to round-trip through the existing source-backed image editor.
