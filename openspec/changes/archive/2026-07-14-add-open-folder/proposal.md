## Why

Markion can currently establish a file-tree workspace only as a side effect of opening or saving a Markdown file. Users need a direct way to choose a folder as the workspace root, including folders where they have not yet opened a document.

## What Changes

- Add an **Open Folder** item immediately after **Open** in both the in-window and native **File** menus.
- Let the user choose one directory and make it the file-tree workspace root without replacing or modifying the active document.
- After a folder is selected, reveal the left sidebar, switch it to **Files**, and scan the selected folder in the background using the existing Markdown-only tree behavior.
- Preserve the selected workspace root while opening Markdown files contained within it; retain the existing document-parent fallback when opening a file outside the current workspace.
- Localize the new menu item, directory prompt, success/cancel/failure feedback, and cover the behavior with focused tests.
- Preserve the existing bounded file-tree rendering and off-main-thread scan invariants.

Non-goals: multi-root workspaces, recent-folder history, workspace persistence across launches, opening every file in a folder as tabs, or assigning a new keyboard shortcut.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `workspace`: Add explicit folder selection as a way to establish the Markdown file-tree workspace and define how that root interacts with document opening.
- `ui-i18n`: Require all new Open Folder labels, prompts, and status feedback to use the existing localization layer.

## Impact

- `src/main.rs`: new GPUI action and handler, directory picker flow, workspace-root update rules, File-menu wiring, sidebar activation, and asynchronous scan result handling.
- `src/i18n.rs`: new exhaustive message keys and translations for every supported interface language.
- Tests around folder selection, workspace-root preservation, menu wiring, localization completeness, cancellation, and scan failure.
- No new dependency, storage format, public API, or Markdown derived-state change is expected.
