## Why

The Files sidebar currently identifies folders with the text label `dir` and Markdown files with `md`, which is visually noisy and less familiar than the icon treatment used by mainstream desktop editors. Replacing those labels with clear icons makes the file tree easier to scan without changing how it behaves.

## What Changes

- Replace the file-tree row prefix text badges with distinct folder and Markdown-file icons, including separate folder icons for expanded and collapsed folders.
- Let folder rows toggle expanded/collapsed state on click while file rows keep opening Markdown files.
- Keep file and folder names on one line; support horizontal scrolling when a tree row is wider than the visible sidebar.
- Tighten row spacing so the tree reads like a conventional editor sidebar.
- Keep existing row indentation, active/selected styling, filtering, create/rename/delete/refresh behavior, and the bounded row-rendering limit.
- Non-goal: no new file types, drag-and-drop, theme system changes, or changes to the Markdown-only tree scan.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `workspace`: file-tree rows should use recognizable icons for folders and Markdown files instead of text badges, support folder expand/collapse, keep names on one line, and preserve existing file-tree behavior.

## Impact

- Affected code: `src/main.rs` file-tree row rendering.
- Affected specs: `workspace`.
- APIs/dependencies: no public API changes and no new dependencies expected.
- Invariants: preserve the file tree's bounded rows per frame; no impact to Markdown parsing, derived-state caching, syntax highlighting memoization, cached text handles, or undo snapshots.
