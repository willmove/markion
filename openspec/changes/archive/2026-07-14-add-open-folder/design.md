## Context

The file tree already has a `workspace_root`, performs `FileTree::scan` on the background executor, renders only Markdown-bearing paths, and caps the number of rows rendered per frame. Today `update_workspace_root_from_document` always derives that root from the active document's parent, so an untitled welcome document cannot establish a workspace and opening a nested document can narrow the tree root.

Markion exposes each File command twice: as a GPUI native `MenuItem` installed by `install_menus`, and as an in-window dropdown button that calls the same app handler. All visible labels, prompts, and statuses are exhaustive `Msg` translations.

This change touches command wiring, workspace selection, asynchronous scan completion, sidebar preferences, and localization. It does not affect document text or any versioned/cached Markdown derived state.

## Goals / Non-Goals

**Goals:**

- Offer Open Folder immediately after Open in both File-menu surfaces.
- Select exactly one directory without replacing, saving, or otherwise mutating the active document.
- Make the selected directory the workspace root, reveal the sidebar on Files, and scan without blocking the GPUI thread.
- Keep a broader selected root while documents opened from within it become active.
- Preserve current behavior when a document outside the current root is opened.
- Provide localized prompt and status feedback, including cancellation and scan failure.

**Non-Goals:**

- Multi-root workspaces, recent folders, root persistence across launches, or OS shell integration.
- Opening a folder's files as tabs or changing which file extensions the tree displays.
- A new shortcut or command-palette surface.
- Changes to Markdown parsing, preview/outline caches, syntax highlighting, or text-handle reuse.

## Decisions

### 1. Use one shared `OpenFolder` GPUI action and app handler

Declare an `OpenFolder` action beside `OpenDocument`, register its handler on the app view, add it immediately after Open in `install_menus`, and add the corresponding in-window dropdown item. The handler uses `prompt_for_paths` with `files: false`, `directories: true`, and `multiple: false`.

This follows the existing command architecture and keeps native and in-window behavior identical. A separate platform-specific folder picker was rejected because GPUI already supplies the cross-platform prompt and Markion targets Windows, macOS, and Linux from the same source.

### 2. Opening a folder changes workspace UI state, not document state

On a successful directory choice, normalize the chosen path, close the active menu, clear tree-root-specific selection/collapse/scroll state when the root changes, set the workspace root, reveal the sidebar, set `SidebarTab::Files`, persist the existing sidebar preferences, and schedule a tree scan. The active tab, document dirty state, undo history, selection, and derived caches remain untouched.

Cancellation leaves the root, file tree, sidebar state, active document, and status-sensitive editing state unchanged except for a localized cancellation message. No discard confirmation is needed because Open Folder never replaces a document.

An alternative that opens the first Markdown file found was rejected because it would make folder selection destructive or surprising and would not work for an empty folder.

### 3. Preserve the current root for documents contained by it

Refine document-driven root updates so that an opened document whose normalized path is inside the current file-tree root does not replace that root with its immediate parent. If the opened document is outside the current root, keep the existing fallback: rebase the workspace to the document's parent and rescan.

This requires no persisted root-source flag and gives both document-derived and explicitly selected roots consistent behavior. Always pinning an explicit root until another Open Folder command was rejected because File → Open on an unrelated document would leave the Files panel disconnected from the active document and would require additional persisted/session state.

### 4. Keep scanning asynchronous and reject stale results

Continue scanning on the background executor. Carry the requested root through the result and apply it only if it still matches the app's current normalized `workspace_root`; this prevents a slow earlier scan from overwriting a newer folder selection. A successful scan installs the new `FileTree` and reports the selected folder. A failed scan reports a localized failure and does not mutate the active document.

The tree continues to use the existing Markdown-only traversal and bounded row rendering. Scanning synchronously before updating the UI was rejected because large directories would freeze the first interaction after choosing a folder.

### 5. Extend the exhaustive localization surface

Add message keys for the Open Folder menu label, directory-picker prompt, opening/success/cancel/failure statuses, and translations for every `Language` currently supported. Reuse `t`/`tf` and the existing all-messages localization test so missing translations remain compile- or test-time failures.

## Risks / Trade-offs

- [Path aliases or differing case on Windows could defeat containment checks] → Compare canonicalized paths where available and fall back to the original path, following the existing comparable-path pattern.
- [Overlapping scans can complete out of order] → Associate completion with its requested root and ignore stale results.
- [A very large folder can take time to appear] → Keep traversal off the UI thread and retain bounded rendering; use localized opening/failure/success status feedback.
- [Opening an external document changes a user-selected root] → This is intentional compatibility with existing document-parent behavior and avoids a Files panel unrelated to the active document.
- [Related active file-tree changes may touch the same files/spec capability] → Keep this implementation scoped to command/root selection and re-run the full test suite when applying.

## Migration Plan

No data migration is required. Implement the action, root-selection behavior, localization, and tests as an additive change. Rollback consists of removing the new action/menu entries and restoring the previous unconditional document-parent root update; no stored workspace-root value needs cleanup.

## Open Questions

None. The selected folder is session-only, the sidebar automatically opens on Files, and no shortcut is added.
