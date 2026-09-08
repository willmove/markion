## Why

Focus mode dims surrounding paragraphs in Source editing but has no visible effect in Visual Edit. The same persisted preference should provide an equivalent focus cue on both editable surfaces.

## What Changes

- Apply focus dimming to Visual Edit content outside the current source-backed editing block, including nested editable leaves and source islands.
- Follow caret movement, selection direction, view/document changes, and the existing preference toggle without modifying Markdown.
- Add regression coverage for actual rendered focus presentation and cache preservation.

Non-goals: changing Source focus behavior, typewriter positioning, Read/Split preview presentation, exports, or preference storage.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `chrome-platform`: Specify Visual Edit focus presentation and interaction-only updates alongside existing focus/typewriter behavior.

## Impact

The root GPUI application's Visual Edit rendering and tests. Preserve per-version shared Markdown/visual caches, memoized highlighting, cached source text, document history, and virtualized rendering. No dependencies or storage migration.
