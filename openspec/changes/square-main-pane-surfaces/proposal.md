## Why

The editor and preview content surfaces currently use rounded rectangles, which adds visual decoration around the primary document area and clashes with the compact pane chrome. Square-corner surfaces make the main workspace cleaner and visually continuous across view modes.

Non-goals: this change does not alter pane spacing, borders, colors, scrolling, resizing, Markdown rendering, editing behavior, or theme definitions.

## What Changes

- Render the source editor surface with square corners in Edit and Split Preview modes.
- Render the visual editor surface with square corners in Visual Edit mode.
- Render the preview surface with square corners in Split Preview and Read modes.
- Preserve the existing surface background fill, border, padding, scrollbars, drag-and-drop handling, and mode-specific layout.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `chrome-platform`: Require the primary source editor, visual editor, and rendered preview surfaces to use square corners consistently across view modes.

## Impact

- Affected code is expected to be limited to the GPUI main-pane surface styling in `src/app/root_view.rs`.
- No public APIs, persisted data, localization, dependencies, or theme palette values change.
- Document-version caches, memoized highlighting, cached text handles, and per-tab scroll state remain untouched.
