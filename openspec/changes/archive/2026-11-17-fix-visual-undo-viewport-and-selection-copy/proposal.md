## Why

Undo in Visual Edit can jump a scrolled document back to its beginning, losing the user's editing location. Selected text also lacks a Copy command in the Visual Edit context menu.

## What Changes

- Preserve the visible editing location during Visual Edit undo/redo and reveal the restored caret only when necessary.
- Add a localized Copy command for a non-empty Visual Edit text selection, matching the existing Copy action.
- Add GPUI regressions for rendered viewport stability and non-mutating clipboard behavior.

Non-goals: changing history grouping, clipboard formats, or other editor modes. Preserve versioned Arc caches and source-backed editing.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Visual Edit history viewport preservation and selection Copy in its context menu.

## Impact

Application history restoration, visual list synchronization, block context menu actions, and GPUI tests. No dependencies or persisted format changes.
