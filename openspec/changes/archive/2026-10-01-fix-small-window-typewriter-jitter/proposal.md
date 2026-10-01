## Why

Typing in typewriter mode visibly displaces the page in a small window. Existing centering checks allow several refinement frames and can miss transient jumps or oscillation during typing. Consecutive Enter presses also alternate between keeping the caret on the preceding text row and creating a real blank row, which can make all content above the caret appear to disappear every other press.

## What Changes

- Reproduce the reported small-window behavior through the actual GPUI editing and drawing path.
- Stabilize typewriter positioning during successive frames and consecutive edits, including wrapped content taller than the viewport.
- Give the first terminal newline its own source-backed Visual Edit whitespace row so every Enter uses the same caret layout.
- Add regression coverage for intermediate positions as well as final centering.

Non-goals: animated scrolling, new preferences, changes to inserted Markdown bytes, unrelated workspace changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `chrome-platform`: Require stable typewriter positioning in small viewports during typing and idle refinement.
- `markdown-editing`: Require every terminal Enter after unquoted prose to create the source-backed blank caret row immediately.

## Impact

GPUI editor scrolling, visual caret measurement, terminal Visual Edit block ownership, and app tests. Preserve per-version derived Markdown Arcs, memoized highlighting, and cached text handles. No persisted format, dependency, or document-content changes.
