## Why

Typewriter mode currently appears enabled but does not reliably keep the active caret row centered: source editing derives its target from hard newline counts and a fixed ten-line offset, while Visual Edit only performs ordinary visibility reveal. Soft-wrapped lines, viewport resizing, reflow, and edits near the document end therefore leave the caret away from the promised vertical center or make the mode appear to do nothing.

## What Changes

- Center the active caret row from measured layout geometry and the actual viewport height in Edit mode and the source pane of Split Preview mode.
- Apply equivalent caret-centered following in Visual Edit using its virtual-list caret geometry, while preserving its existing coarse reveal/refinement behavior.
- Defer and retry centering until current-version layout and scroll extents are available after edits, reflow, tab/view changes, or first render.
- Provide presentation-only leading and trailing scroll room so the first and final editable rows can remain centered, without changing document text or derived Markdown caches.
- Preserve normal caret reveal when typewriter mode is disabled and preserve Split Preview source-to-preview sync when the centered source pane is the scroll driver.

**Non-goals:** redesigning the typewriter preference or shortcut, adding animation/smoothing, changing Read mode, changing focus mode, or changing user-authored Markdown and its cached derived state.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `chrome-platform`: make the existing typewriter-mode contract explicit for all editable surfaces, measured soft-wrapped rows, layout changes, and document boundaries.

## Impact

- Affected application areas: per-tab editor presentation state, source editor layout/scroll reconciliation, Visual Edit caret-follow logic, view/tab/typography transitions, and Split Preview sync-scroll coordination under `src/app/`.
- Tests will cover pure centering calculations plus GPUI integration behavior for source and Visual Edit surfaces, including soft wrap, end-of-document padding, disabled mode, and cache/document invariants.
- No public API, file-format, localization, dependency, or preference-schema changes are expected.
- The document-version `Arc` caches, syntax-highlighting memoization, cached text handles, and undo snapshot boundaries remain unchanged; added padding and pending-centering state are presentation-only.
