## Why

The Files sidebar already exposes file and folder management through a right-click menu, but it does not provide a quick way to inspect entry details such as type, path, size, and modified time. Users also commonly need to copy a path while working across editor, terminal, and file manager workflows.

## What Changes

- Extend the file-tree context menu for files and folders with a localized **Properties** action.
- Add localized **Copy Path** and **Copy Relative Path** actions for files and folders.
- Keep the existing context-menu management actions available where appropriate: Delete and Rename for files/folders, plus existing Open, Open in New Tab, New File, New Folder, Show in System File Manager, Refresh, and Filter Files actions.
- Show entry properties in an in-app dialog or equivalent app surface with path, entry kind, size, and modified timestamp; folders should report recursive size only if it can be computed without blocking the UI.
- Non-goals: moving entries, showing non-Markdown files in the tree, or replacing the existing inline create/rename prompt.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `workspace`: File-tree right-click menus shall include Properties and path-copy actions in addition to existing entry management actions.
- `ui-i18n`: New context-menu labels, properties dialog text, and status feedback shall be localized through the i18n layer.

## Impact

- Affected code is expected in `src/main.rs`, `src/i18n.rs`, and possibly `src/storage/file_tree.rs` if property metadata needs a helper.
- The change touches file-tree UI state and filesystem metadata reads, while preserving bounded row rendering.
- Markdown derived-state caching, syntax highlighting memoization, cached text handles, and undo snapshots are not affected.
