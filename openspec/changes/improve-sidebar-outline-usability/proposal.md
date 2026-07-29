## Why

The visible sidebar currently starts below the multi-document tab band, leaving an unused strip above Files/Outline, while long outlines are overly spaced and cannot be scrolled to reach hidden headings. Reclaiming that strip and making the outline compact and scrollable will improve both space use and navigation in large documents.

## What Changes

- Extend the visible sidebar from directly below the menu bar through the full workspace height, so its Files/Outline tabs occupy the area that is currently an empty tab-band spacer.
- Keep document-tab controls aligned with the document workspace to the right of the sidebar and preserve the active-tab-to-document connection.
- Reduce outline-row vertical spacing to a compact density comparable to the file tree while retaining hierarchy indentation and readable click targets.
- Give the outline its own bounded vertical scrolling region so mouse-wheel and trackpad scrolling can reach every heading when content exceeds the viewport.
- Preserve outline click navigation, active-section highlighting, per-document derived-state caching, and the file tree's existing bounded rendering and scrolling behavior.

Non-goals: this change does not add outline folding, virtualize outline rows, change heading derivation, alter sidebar persistence, or redesign document-tab interactions.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `chrome-platform`: Define the sidebar and document-tab band as adjacent workspace columns so the visible sidebar occupies the full height below the menu bar.
- `tables-outline`: Require compact outline rows and vertical scrolling that exposes all headings in an overflowing outline.

## Impact

- GPUI layout and styling in `src/app/root_view.rs` and `src/app/editing.rs` will change, with a dedicated outline scroll handle added to application state and initialization as needed.
- Focused regression coverage will be added in `src/app/tests.rs`; existing tab, sidebar resize, file-tree scroll, and outline navigation behavior must remain intact.
- No public APIs, persisted data, localization strings, dependencies, Markdown parsing, or cache ownership rules change.
