## Context

The Files sidebar row rendering lives in `src/main.rs` and currently prefixes each visible row with a text marker: `dir` for `FileTreeEntryKind::Directory` and `md` for `FileTreeEntryKind::File`. The file-tree model and scan path already distinguish directories from Markdown files, filter the tree, preserve selected/current row state, and cap visible rows per frame.

This change is a presentation refinement on the existing row view. It does not touch file discovery, document opening, persistence, Markdown parsing, or any derived document cache.

## Goals / Non-Goals

**Goals:**

- Render folder rows and Markdown-file rows with compact, recognizable icons instead of literal `dir` / `md` text badges.
- Distinguish expanded and collapsed folders visually.
- Let directory rows toggle their descendants without making directory rows open documents.
- Keep file and folder names on one line and make the row list horizontally scrollable when names exceed the sidebar width.
- Keep indentation, active/selected colors, hover behavior, filtering, and the visible-row cap predictable.
- Use GPUI primitives already present in the file so the change stays local and dependency-free.

**Non-Goals:**

- No drag-and-drop or move UI.
- No file-type expansion beyond the existing Markdown-only tree.
- No changes to theme definitions, i18n strings, storage, or document caches.

## Decisions

1. Use small GPUI-drawn icons rather than image assets or a new icon dependency.

   Rationale: the app already renders this panel with simple GPUI elements. A small helper can draw a folder/file silhouette with borders and theme colors, avoiding new assets, font dependencies, platform differences, or localization concerns.

   Alternative considered: Unicode emoji or text symbols. Rejected because emoji rendering varies by platform and can be visually loud in a desktop editor sidebar.

2. Track collapsed folders in UI state, not in the file-tree storage model.

   Rationale: expand/collapse is view state. `FileTree` should remain the source for the scanned Markdown-only hierarchy, while `MarkionApp` keeps a set of collapsed directory paths and filters descendants at render time. New scans prune vanished collapsed paths, and switching workspaces clears stale collapsed state.

   Alternative considered: add expansion state to `FileTreeEntry`. Rejected because it mixes transient UI state into the storage/scanning layer and would make background scans responsible for preserving view state.

3. Keep the icon inside the existing row element.

   Rationale: preserving the current row as the click target keeps selection and opening behavior unchanged. The implementation only replaces `format!("{marker} {}", entry.name)` with a flex row containing an icon element and the filename text.

   Alternative considered: add a separate icon column. Rejected because it would require broader layout changes and could disturb the compact sidebar density.

4. Preserve current file-tree data flow, with a render-time visibility pass.

   Data flow remains:

   `FileTree::scan` -> filtered/limited entries -> hide descendants of collapsed directories -> row map in `file_tree_panel_body` -> icon chosen from `FileTreeEntryKind` and expansion state -> row click opens Markdown files or toggles directories.

   No document text, derived Markdown state, syntax highlighting cache, text handle cache, undo snapshot, or scan/filter logic is affected.

5. Prefer nowrap + horizontal scrolling over wrapping or truncation.

   Rationale: file trees in desktop editors keep each entry as one scannable row. A horizontal scrollbar is less disruptive than increasing row height or hiding the meaningful tail of long filenames.

   Alternative considered: truncate with ellipsis. Rejected because the user explicitly asked for the full name to remain on one line.

## Risks / Trade-offs

- Icon contrast may be too subtle on some themes -> derive icon colors from the active row text color and existing palette colors, and verify visually in at least one light and one dark built-in theme if possible.
- Hand-drawn GPUI icons can look less polished than a full icon set -> keep shapes simple and conventional: folder tab + body, document sheet with folded corner/Markdown mark.
- Horizontal scrolling can make row backgrounds wider than the visible sidebar -> keep row widths bounded to estimated content width and use the existing visible-row cap so rendering remains bounded.
