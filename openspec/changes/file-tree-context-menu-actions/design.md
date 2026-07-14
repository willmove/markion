## Context

The Files panel currently builds an always-visible filter field plus a toolbar row for create file, create folder, rename, delete, and refresh. Row click already selects entries and opens Markdown files; keyboard actions already exist for the same tree operations. This change moves the visible controls into a contextual surface so the tree itself becomes the primary content.

The pending `use-icons-in-file-tree` change also touches row rendering and folder state. If both changes are implemented before archiving, apply this change after rebasing on that row-rendering work so the context menu composes with icon rows and folder expand/collapse.

## Goals / Non-Goals

**Goals:**

- Remove the persistent filter input and action toolbar from the Files panel.
- Show a right-click context menu for file rows, folder rows, and blank/background tree space.
- Offer context-appropriate operations: open/open in new tab for files, create inside folders or workspace space, rename/delete for selected entries, refresh, and reveal in the system file manager.
- Keep existing keyboard shortcuts and command handlers where they still make sense.
- Route every new label and status message through `src/i18n.rs`.

**Non-Goals:**

- No drag-and-drop moves.
- No broad redesign of the sidebar tabs.
- No new non-Markdown file visibility.
- No changes to document parsing, derived Markdown caches, syntax highlighting, or undo snapshots.

## Decisions

1. Model the context menu as app state rendered by `sidebar_view`.

   Rationale: the existing in-window menu bar uses app state plus positioned `Div`s. Reusing that pattern avoids new dependencies and keeps the menu themeable with the current palette. Right-click handlers set the target path/kind and menu position, while click-outside or action execution clears it.

   Alternative considered: native OS context menus. Rejected because GPUI's in-app menu pattern is already present and gives consistent styling/localization across platforms.

2. Keep file-tree operations routed through existing action helpers.

   Rationale: create, rename, delete, and refresh already update the tree, status bar, and selected path. Context menu items should call those same helpers after setting `selected_tree_path` and choosing the proper parent path, rather than duplicating filesystem behavior.

   Alternative considered: implement separate context-menu filesystem paths. Rejected because it would increase the chance of drift from keyboard/menu behavior.

3. Add targeted helpers for new context-only operations.

   - Open: use the same document-opening path as a left-click file row.
   - Open in new tab: open a Markdown file in a new tab even if a future left-click mode changes.
   - Show in system file manager: use platform-specific `std::process::Command` calls (`explorer /select,`, `open -R`, `xdg-open`) and report failures through localized status text.
   - Filter files: invoke a transient filter-entry affordance from the context menu without keeping a persistent filter box in the panel.

4. Preserve file-tree data flow and performance boundaries.

   Data flow remains:

   `FileTree::scan` -> filtered/visible entries -> bounded row render -> context menu action -> existing file-tree command/open/reveal helper -> tree refresh/status update.

   The context menu is a small overlay and does not alter document text, derived Markdown state, syntax highlighting cache, text handle cache, or undo snapshots.

## Risks / Trade-offs

- Right-click behavior varies slightly by platform -> handle `MouseButton::Right` directly in GPUI rows/background and keep keyboard shortcuts as a fallback.
- Removing visible controls can reduce discoverability -> include the core actions in the context menu for rows and blank space, and keep existing menu/shortcut references where already present.
- Revealing in the system file manager can fail when platform commands are unavailable -> surface a localized failure status and leave editor state unchanged.
- Multiple active OpenSpec changes touch the Files panel -> implement after or alongside `use-icons-in-file-tree` to avoid row-rendering conflicts.
