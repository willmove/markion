## Why

Large Markdown documents are awkward to navigate when the editor and preview panes only feel scrollable by mouse wheel, and the current pane padding leaves too much unused space around text. Making pane scrollbars visibly draggable and tightening the editor chrome lets users inspect more source and rendered content at once.

Non-goals: this change does not alter Markdown parsing, editing semantics, view-mode switching behavior, caching, export output, or theme color definitions.

## What Changes

- Make the editor pane and preview pane expose visible, right-side vertical scrollbars that can be dragged for large documents.
- Keep existing wheel/trackpad scrolling and per-tab scroll-position preservation.
- Tighten the main editor/preview/sidebar layout spacing so pane gaps and outer padding are approximately 15% of the current visual spacing.
- Preserve usable drag targets for the editor/preview split handle and sidebar resize handle while reducing their visible footprint.
- Keep single-pane Edit and Read modes full-width within the remaining workspace.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `chrome-platform`: Adds requirements for visible draggable pane scrollbars and denser application chrome around the editor, preview, and sidebar panes.
- `markdown-editing`: Clarifies that pane scroll positions remain isolated per tab and continue to be preserved while using visible scrollbars.

## Impact

- Affected code is expected to be concentrated in `src/main.rs`, especially the GPUI layout for `editor-scroll`, `preview-scroll`, sidebar/split resize handles, and pane padding.
- No new runtime dependencies are expected.
- The change touches rendering/layout only and must preserve derived Markdown state caching, memoized highlighting, cached text handles, and per-tab scroll state.
