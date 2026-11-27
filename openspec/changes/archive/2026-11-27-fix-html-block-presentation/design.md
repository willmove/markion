## Context

The shared HTML preview builder resolves block alignment before GPUI lays out the text. Text with authored breaks or scripts uses a fragment layout whose full-width flex rows currently start at the left. The Visual Edit HTML editor puts the rendered parts and the lazy source field into a shared collapsible container.

## Goals / Non-Goals

- Render aligned HTML with consistent text geometry and pointer hit targets across the three rendered modes.
- Keep the existing HTML parsing, source mapping, and document version boundaries. Browser-style inline image flow is outside this change.

## Decisions

- Pass alignment from each HTML text part to the existing rich-text fragment rows. Flex justification positions the fragments on each authored or wrapped line together with their selection and link hit targets. Default-aligned text can continue to use the existing shaped-text path.
- Turn off the outer border on the Visual Edit HTML container. Keep its source toggle, the source field styling, and the table renderer's own grid borders.
- Keep alignment and border handling in the view layer. The per-version derived Markdown model and lazy source projection retain their existing ownership.

## Risks / Trade-offs

- Aligned rich text uses the existing fragment wrapping rules. Verify at both full and split pane widths to cover wrapping, positioned script text, and links.
