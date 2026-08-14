## Why

Split Preview currently equates the two panes by whole-document scroll percentage. Markdown source and rendered blocks expand by very different amounts, so headings, images, tables, code fences, and wrapped prose accumulate local drift even when both scrollbars report the same percentage; the panes therefore move together without staying on the same document content.

## What Changes

- Replace global proportional coupling with source-mapped synchronization based on the existing source ranges carried by rendered preview blocks.
- Keep a shared semantic viewport anchor: scrolling either pane maps the visible source location into the other pane and aligns the corresponding rendered/source location, with piecewise interpolation inside blocks and across source gaps.
- Preserve continuous scrolling in both directions, including wheel, trackpad, scrollbar-drag, find/goto, and typewriter-driven editor movement, without feedback loops or follower-pane jitter.
- Handle unmeasured virtual-list rows with a bounded two-phase jump-and-refine path, and suspend remapping while debounced preview blocks are stale rather than forcing a synchronous Markdown parse.
- Preserve the persisted Sync scroll preference, its default-off behavior, per-tab independent scroll state, and independent scrolling outside Split Preview.

Non-goals: changing the preference UI or persistence format, synchronizing horizontal scrolling, adding inline DOM-style source markers, removing preview virtualization, or reparsing Markdown solely to satisfy scrolling.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `chrome-platform`: Replace percentage-based Split Preview coupling with source-mapped semantic alignment and define fallback, stale-preview, and boundary behavior.
- `markdown-editing`: Require per-tab pane scroll state to synchronize by document location while retaining separate GPUI scroll/list state and existing cache invariants.

## Impact

- Affected code: `src/app/appearance.rs` (mapping and reconciliation), `src/app/state.rs` (per-tab synchronization observations/targets), `src/app/root_view.rs` and possibly `src/app/editor_element.rs` (visible geometry handoff), plus focused tests in `src/app/tests.rs`.
- Reuses `PreviewBlock::source_range`, the editor's per-version line-offset/line-height caches, and `ListState` logical offsets/visible row bounds; no new parser output or dependency is required.
- The derived-state `Arc` cache, debounced preview parsing, virtualized preview list, cached text handle, document version, dirty state, and undo/redo history must remain untouched by scroll reconciliation.
