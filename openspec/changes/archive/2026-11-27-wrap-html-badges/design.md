## Context

The HTML parser emits each image as a distinct part and already assigns it a resolved alignment and link. By the time the renderer sees adjacent parts, a paragraph boundary may have produced no visible part of its own.

## Goals / Non-Goals

- Group consecutive images across source whitespace and inline link tags, while respecting authored line and block boundaries.
- Reuse intrinsic image layout, cache lookups, links, and the existing standalone image presentation.

## Decisions

- Retain image parts in their existing order and annotate an image that continues the preceding image line. The parser records block boundaries before they are lost, and an intervening text or table part naturally ends a group.
- Consume grouped images in the shared HTML view using a wrapping flex row. Each child uses the existing compact inline image path with its own dimensions and link. Justification acts independently on each wrapped line, and a single image uses the full-width presentation.
- Keep image descriptor traversal in the existing source order. No changes to document versions, derived-state invalidation, or image load scheduling are required.

## Risks / Trade-offs

- A badge can change size when its image finishes loading. Layout uses the currently visible image or placeholder and reflows when the existing image update arrives.
