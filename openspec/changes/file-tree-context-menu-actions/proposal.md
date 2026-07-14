## Why

The Files sidebar currently spends scarce vertical space on an always-visible filter box and action buttons (`New`, `Dir`, `Ren`, `Del`, `Ref`). These controls make the tree feel more like a form than a desktop editor file explorer, and they duplicate actions that fit more naturally in a right-click context menu.

## What Changes

- Remove the always-visible file-tree filter field and toolbar buttons from the Files sidebar body.
- Add a right-click context menu for file-tree rows and empty tree/background space.
- Move create file, create folder, rename, delete, and refresh into the context menu.
- Add context actions where they fit:
  - files: open, open in new tab, rename, delete, show in system file manager, refresh
  - folders: create file, create folder, rename, delete, show in system file manager, refresh
  - empty/background space: create file, create folder, refresh, show workspace in system file manager
- Keep existing keyboard shortcuts for file-tree actions so power users do not lose access.
- Non-goal: no drag-and-drop moves, no new file types, no global command palette, and no changes to Markdown parsing or document caches.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `workspace`: file-tree operations move from always-visible controls into a contextual right-click menu, with additional file/folder actions where appropriate.
- `ui-i18n`: new context-menu labels and status messages must be routed through i18n.

## Impact

- Affected code: `src/main.rs` file-tree rendering, context-menu state/action handlers, and OS file-manager launcher; `src/i18n.rs` for new labels/status text.
- Affected specs: `workspace`, `ui-i18n`.
- APIs/dependencies: no public API changes and no new dependency expected; system file-manager reveal should use platform-specific commands from the standard library.
- Invariants: preserve the file tree's bounded row rendering; no impact to Markdown derived-state caching, syntax highlighting memoization, cached text handles, or undo snapshots.
