## Why

Long documents currently expose every heading in one permanently expanded outline, which makes deeply nested documents slow to scan and forces unnecessary sidebar scrolling. Users need an obvious way to collapse sections they are not working with and expand them again without losing the existing click-to-jump behavior.

## What Changes

- Render hierarchical outline rows with a disclosure control on headings that have descendants.
- Start each document outline fully expanded; clicking a disclosure control collapses or expands that heading's descendant rows.
- Keep heading-label clicks dedicated to the existing context-aware navigation behavior, including Read-mode preview jumps and active-section highlighting.
- Keep folding as per-document, session-only presentation state that does not modify Markdown, document version, dirty state, history, or the cached derived outline.
- Reconcile folding state when headings change so stale collapsed entries cannot hide unrelated headings.

**Non-goals**: Persisting folded outline sections across application restarts, folding Markdown content in the editor/preview itself, adding keyboard-driven tree navigation, or changing document heading parsing.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tables-outline`: Extend document outline navigation with default-expanded, per-heading collapse and expand behavior while preserving navigation, highlighting, compact spacing, and scrolling.

## Impact

- **Affected code**: outline presentation and interaction in `src/app/root_view.rs`, per-document UI state in `src/app/state.rs`, application interaction helpers, icons/styles if needed, and focused coverage in `src/app/tests.rs`.
- **User experience**: Nested outlines become easier to scan, while clicking a heading label continues to navigate exactly as before.
- **Architecture**: Folding filters the already-cached per-version outline for presentation only; it must not reparse Markdown, duplicate derived outline data, or invalidate preview/outline/stat, syntax-highlight, or cached-text state.
- **Compatibility/dependencies**: No file-format, preference, public API, localization, or dependency changes are required.
