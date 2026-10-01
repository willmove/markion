## Context

The parser already propagates `align="center"` from the HTML paragraph to `HtmlPreviewPart::Image`, and the existing shared `html_preview_block_view` is used by Read mode, Split Preview, and the HTML presentation inside Visual Edit. The image branch currently applies flex justification to a shrink-to-content wrapper, so there is no free horizontal space for centering. See `proposal.md` for the user-visible defect.

The fix is presentation-only: it must not alter `MarkdownDocument`, preview block derivation, image cache claims, authored dimensions, or source ranges. Those derived structures remain cached per document version.

## Goals / Non-Goals

**Goals:**

- Give HTML image parts a full-width layout container before applying their resolved start/center/end alignment.
- Keep the correction shared across Read, Split Preview, and Visual Edit HTML blocks.
- Verify the parser metadata remains correct and the renderer's alignment container is wide enough for the supplied centered image case.

**Non-Goals:**

- No changes to Markdown image alignment, image decoding, image sizing, or HTML parsing.
- No new layout abstraction or dependency.

## Decisions

- **Apply `w_full()` to the HTML image wrapper:** the existing image branch already computes the correct `HtmlAlign`; making that same wrapper full width lets `justify_center()` and `justify_end()` operate against the rendered content width. This is preferable to changing the parser or adding a second wrapper because all three alignment modes stay in one shared path.
- **Keep alignment conditional:** the wrapper remains a flex container only when the parsed image is centered or right-aligned, preserving the current start-aligned behavior while ensuring the conditional container fills the row when active.
- **Test the existing parsed contract plus the shared layout seam:** retain the existing parser test for `<p align="center"><img ...></p>` and add a focused regression assertion around the renderer helper/structure so a future change cannot remove the full-width constraint while leaving parser metadata green.

## Risks / Trade-offs

- [Risk] A full-width wrapper could change spacing for images with no explicit alignment. → Mitigation: apply the width constraint only in the alignment branch; start-aligned images retain their existing intrinsic wrapper behavior.
- [Risk] The test may validate structure rather than rasterized pixels. → Mitigation: run the targeted parser and application tests, and use the shared renderer path that all affected modes call; no alternate mode-specific implementation exists.
