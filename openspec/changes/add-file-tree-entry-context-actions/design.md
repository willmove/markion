## Context

The Files sidebar already has in-app context-menu plumbing in `src/main.rs`: menu target state, target-specific action arrays, localized labels, and handlers that route actions to existing file-tree helpers. Existing actions include Open, Open in New Tab, New File, New Folder, Rename, Delete, Show in System File Manager, Refresh, and Filter Files.

The missing user workflow is inspection and path handoff. A user can manage an entry but cannot quickly view its details or copy its path from the editor. These actions are filesystem/UI concerns and should not touch Markdown document text or derived Markdown caches.

## Goals / Non-Goals

**Goals:**

- Add Properties, Copy Path, and Copy Relative Path to file and folder context menus.
- Preserve the existing Delete and Rename actions for file/folder targets.
- Show entry properties without blocking normal rendering.
- Route all new labels, dialog text, and status feedback through `src/i18n.rs`.

**Non-Goals:**

- Do not add drag-and-drop or menu-driven move support.
- Do not list non-Markdown files in the file tree.
- Do not replace the existing inline name prompt for create/rename.
- Do not compute expensive recursive folder metadata on the render path.

## Decisions

1. Extend the existing `FileTreeContextAction` enum instead of adding a parallel menu model.

   Rationale: The app already chooses menu items by `FileTreeContextTargetKind` and labels them through `file_tree_context_action_label`. Adding actions there keeps behavior consistent with existing context-menu dispatch.

   Alternative considered: create a separate "advanced actions" submenu. This would add UI complexity before the action list is large enough to justify hierarchy.

2. Copy absolute and workspace-relative paths as separate actions.

   Rationale: Absolute paths are useful for terminals and external tools; relative paths are better for Markdown links, issue comments, and repo-local notes. Keeping both visible avoids hidden modifier-key behavior.

   Alternative considered: one Copy Path action that chooses relative paths for workspace entries. This is less predictable and can surprise users when they expect an absolute filesystem path.

3. Store properties as transient app state and render them in an in-app dialog or panel.

   Rationale: Metadata belongs to the selected filesystem entry, not to the document model. A small transient state value can be populated by the menu handler and dismissed independently of editor state.

   The displayed fields should include entry kind, absolute path, workspace-relative path when available, byte size for files, and modified timestamp when available. Folder size may be omitted or marked unavailable unless a non-blocking helper is added.

4. Keep filesystem metadata collection out of row rendering.

   Data flow:

   `FileTree::scan` -> filtered/visible entries -> bounded row render -> context menu action -> metadata/path-copy helper -> status/dialog update.

   The new actions do not mutate document text, document version, preview blocks, outline, stats, syntax highlighting cache, text handle cache, or undo snapshots.

## Risks / Trade-offs

- Metadata reads can fail after the tree is rendered if the file is moved externally -> show localized failure status and refresh only when needed.
- Recursive folder size can be expensive -> do not compute it synchronously in the render path; prefer "unavailable" initially or a spawned task if recursive size is required later.
- Clipboard APIs may fail on some platforms -> report a localized failure status and leave tree selection unchanged.
- More context-menu items can make the menu taller -> keep the additions target-specific and avoid workspace/background-only clutter.

## Migration Plan

No data migration is required. The implementation is additive and can be rolled back by removing the new context actions, metadata state, and i18n messages.

## Open Questions

None.
