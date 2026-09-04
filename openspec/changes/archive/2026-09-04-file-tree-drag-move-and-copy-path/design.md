## Context

The Files sidebar already scans a workspace, lists supported files and folders, and exposes create / rename / delete / refresh through a right-click menu. `FileTree::move_entry` exists and is covered by a unit test, but no UI calls it — the `workspace` spec still says moving via the UI is not supported. Tab-bar context menus can copy an open file's path; file-tree menus cannot.

GPUI already has the drag primitives this change needs: Visual Edit uses `on_drag` / `on_drop::<DraggedVisualBlock>`, and pane splitters use typed drag values. File-tree rows currently handle left `mouse_up` (open / toggle) and right `mouse_up` (context menu). External OS file drops open Markdown on the editor/preview panes only; the tree is not an `ExternalPaths` target.

## Goals / Non-Goals

**Goals:**

- Let the user left-drag a listed file or folder onto a folder, onto a file inside a folder, or onto the workspace root and have it move on disk.
- Keep click-to-open and click-to-toggle working when the pointer does not start a drag.
- Remap open tabs after a successful move the same way rename does.
- Add Copy Path and Copy Relative Path to file and folder context menus, writing platform-normal paths to the clipboard.
- Route every new label and status string through `src/i18n.rs`.
- Preserve bounded file-tree row rendering and leave derived Markdown caches, syntax-highlight memoization, cached text handles, and undo snapshots alone except for rename-style tab remapping.

**Non-Goals:**

- Multi-select, copy-on-drop, or cross-workspace moves.
- Auto-expanding a collapsed folder on hover.
- OS-originated drops onto the tree (those stay editor/preview open-file drops).
- A Properties dialog (that remains out of scope; see leftover `add-file-tree-entry-context-actions`).
- New file types, scan-filter changes, or a persistent move toolbar.

## Decisions

1. **Internal drag type, not `ExternalPaths`.**

   Add `DraggedFileTreeEntry { path: PathBuf, kind: FileTreeEntryKind }` next to the existing drag-value types in `src/app/mod.rs`. File and folder rows call `on_drag` with that value; folder rows, file rows, the workspace-root header, and blank tree space call `on_drop::<DraggedFileTreeEntry>`. File-row drops resolve to the file's parent folder.

   Rationale: GPUI keys drag/drop by type. Reusing `ExternalPaths` would mix OS open-file drops with in-tree moves. A dedicated type keeps the two flows isolated.

   Alternative considered: a dedicated drag handle on each row (Visual Edit's ⠿). Rejected because the user asked to drag the file or folder itself, matching desktop file explorers.

2. **A drag must not also click.**

   File-tree rows today open or toggle on left `mouse_up`. When `on_drag` starts, set a short-lived `file_tree_drag_active` flag (or equivalent) so the subsequent `mouse_up` skips open/toggle. Clearing the flag on drop or drag cancel.

   Rationale: without this, dragging a Markdown file would move it and then open it, and dragging a folder would toggle collapse.

   Alternative considered: switch left-click open to `on_click`. Rejected as a wider behavior change; a drag-session flag is local to this feature.

3. **Valid drop targets are folders, files-as-parent, and the workspace root.**

   - Folder row → move into that folder (collapsed folders still accept a drop; they do not auto-expand).
   - File row → move into that file's parent folder (Explorer-style "drop anywhere in the folder's contents").
   - Workspace-root header or blank Files-panel space → move into `workspace_root`.
   - The dragged entry itself, a descendant of a dragged folder, or the entry's current parent (including a sibling file) → no filesystem change; optional muted status for the clearly invalid cases (self / descendant). Same-parent drops are silent no-ops.

   Rationale: aiming only at the folder row is fiddly when the folder already contains files. Dropping onto any row under the destination folder means "move into this folder."

   Alternative considered: ignore file-row drops. Rejected after real-window testing showed it made in-tree moves unnecessarily precise.

4. **Harden `move_entry` instead of inventing a second move API.**

   Today's `FileTree::move_entry` calls `unique_child_path` and does not run `ensure_existing_path_within_root`. Tighten it to:

   - require the source to exist inside the workspace root and not be the root;
   - require the destination parent to be the root or a directory inside it;
   - refuse moving a folder into itself or a descendant;
   - refuse when the destination already has an entry of the same name (`AlreadyExists`) instead of silently renaming.

   The UI calls this API after the same checks so status messages can be specific. Existing unique-name helpers stay for create/rename.

   Rationale: unique-on-collision is surprising for a drag ("I dropped `notes.md` and it became `notes 1.md`"). Create/rename already ask for a name; drag does not.

   Alternative considered: keep unique-name move and only validate in the UI. Rejected because an unguarded `move_entry` is a footgun if anything else starts calling it.

5. **Refuse the move when any affected open tab is dirty.**

   Before touching disk, if any open tab's path equals the source or (for a folder) is under the source, and that tab is dirty, abort with a localized save-first status (new `StatusSaveBeforeMove`, parallel to `StatusSaveBeforeRename`). Clean tabs remapped after a successful `fs::rename` reuse the rename path: reopen documents from the new path, retarget image tabs, drop undo/redo that pointed at the old document instance.

   Rationale: rename already refuses a dirty active document. A folder move can orphan several dirty tabs at once; blocking is safer than silently remapping in-memory buffers onto new paths.

   Alternative considered: move on disk and keep dirty buffers attached to the new path without reload. Rejected because it diverges from rename and can lose the disk/buffer relationship.

6. **Copy Path / Copy Relative Path are file- and folder-menu items only.**

   Add `FileTreeContextAction::CopyPath` and `CopyRelativePath` to the file and directory action arrays (not the workspace/blank-space menu). Place them after Show in System File Manager and before Refresh.

   - Absolute: write the platform-normal display path (`comparable_document_path` / existing chrome-platform form) via `cx.write_to_clipboard`, reuse `StatusCopiedPath`.
   - Relative: `strip_prefix(workspace_root)` using platform separators; on failure, localized status and no clipboard write. New `StatusCopiedRelativePath`.

   Rationale: the tab-bar already copies an absolute path this way. Relative paths are only meaningful for tree entries that live under the current workspace root.

   Alternative considered: also copy the workspace root from the blank-space menu. Out of scope; the request is "a file or folder."

7. **Hover highlight only; no extra tree rows.**

   While a `DraggedFileTreeEntry` is over a valid folder or the root header, reuse `palette.active_bg` on that target. The drag preview is a small name-only ghost from the `on_drag` builder (`cx.new(|_| …)`), same Empty-or-label pattern as other drags. No extra scan, no extra rows beyond the existing 300-row cap.

   Data flow:

   `row on_drag` → `DraggedFileTreeEntry` → `on_drop` → validate → dirty-tab check → `FileTree::move_entry` → refresh tree + remap tabs → status.

   Copy path: context action → path format helper → clipboard + status. No document version bump.

## Risks / Trade-offs

- **[Left-click and drag share the same row]** → Drag-session flag must suppress `mouse_up` open/toggle; tests should cover "drag started ⇒ click does not fire."
- **[Name collision after a silent unique rename]** → `move_entry` now fails instead; the user must rename or pick another folder.
- **[Moving a folder with open dirty descendants]** → Whole move is refused; no partial moves.
- **[Windows path form]** → Clipboard strings go through the existing normal-form helper so Copy Path does not leak `\\?\`.
- **[GPUI drop hover state]** → If `on_drag_move` targeting proves awkward, fall back to drop-without-hover-highlight rather than adding a hit-test scan on every pointer move.
- **[Stale sibling change `add-file-tree-entry-context-actions`]** → That change also adds Copy Path plus Properties and is unimplemented. This change owns path copy. Do not implement Properties here; reconcile or drop the leftover change later.

## Migration Plan

- No user-data migration. Existing workspaces and sessions are unchanged until the user drags or copies.
- On archive, update the `workspace` spec Purpose paragraph that currently says drag-and-drop moves are not part of the capability.
- Rollback is a revert of the UI wiring and the `move_entry` guard tightening; create/rename/delete paths stay intact.

## Open Questions

- None blocking implementation. Expand-on-hover can be a follow-up if collapsed folders still feel too precise to aim at.
