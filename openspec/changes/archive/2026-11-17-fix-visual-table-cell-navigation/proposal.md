## Why

Visual Edit Up/Down leaves a GFM table at the active cell's line boundary instead of reaching the next cell in the same column. Empty cells also lack a reliable click target and a painted caret, preventing normal editing.

## What Changes

- Keep vertical navigation within the current cell's wrapped lines, then enter the adjacent logical row in the same column; leave the table only at its outer row boundaries.
- Make cell interiors and padding clickable without overriding precise text hit testing or table controls.
- Give empty cells source-backed caret and navigation geometry without inserting placeholder bytes into Markdown.
- Add GPUI regressions for the reported formatted three-column table, empty rows, selection, wrapping, input/undo, and boundary handoff.

Non-goals: table source toggles, implicit row creation, HTML-table editing, or a new editing model.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tables-outline`: define same-column vertical traversal and empty-cell pointer/caret behavior for Visual Edit GFM grids.

## Impact

Visual table rendering, source-mapped navigation snapshots, and application input/layout tests. No new dependencies or persisted format changes. Navigation and clicks preserve document version, dirty state, history, and the existing per-version shared Markdown caches.
