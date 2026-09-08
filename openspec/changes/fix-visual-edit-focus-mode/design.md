## Context

Source and Visual Edit use separate rendering paths. Visual Edit already resolves one caret-owning source-backed row and flattens quoted/list leaves into rows. Its block-view factory returns a GPUI Div, providing a direct presentation regression-test seam without native desktop automation.

## Goals / Non-Goals

**Goals:** visibly dim non-current Visual Edit blocks, follow the canonical caret, cover every block-rendering branch, preserve cached document state.

**Non-Goals:** modify source editing, typewriter scrolling, preview/export styling, preference schema, or Markdown derivation.

## Decisions

- Use the existing unique caret-owner lookup (including selection direction and document-tail handling) instead of a second paragraph parser or selection-intersection heuristic.
- Apply opacity once to the complete Visual Edit row returned by the common block factory. This covers rich text, mixed inline atoms, tables/code/HTML, and early-return conservative source islands without duplicating color logic. Flattened quoted/list leaves dim independently.
- Keep authored theme colors, with a fixed 0.4 opacity on non-current rows and full opacity otherwise. Source's theme-specific muted text colors remain unchanged. Whole-row opacity also dims decorations and images, producing a consistent visual focus cue.
- Data flow: persisted app focus flag + active tab cursor -> existing cached visual block ownership -> row presentation -> GPUI paint. No derived cache invalidation, reparsing, document version bump, or persistent per-row focus state.
- Regression tests inspect opacity on Divs returned by the actual production block factory, exercise caret/selection changes and source islands, and verify document/cache identity. Existing GPUI interaction tests cover pointer routing and caret ownership.

## Risks / Trade-offs

- Early-return source islands could bypass styling -> use a single wrapper around the existing content factory.
- Confusing selection anchor with caret -> reuse existing unique caret ownership.
- Theme/atom colors could bypass inherited text colors -> use subtree opacity.
- Native visual appearance is not asserted by a pixel screenshot -> verify actual GPUI row styles and exercise headless window interactions.

## Migration Plan

No migration. The existing preference applies immediately; reverting the render wrapper restores prior presentation.

## Open Questions

None.
