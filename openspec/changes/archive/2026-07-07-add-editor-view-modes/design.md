## Context

The app already has a `ViewMode` state in the root GPUI app, with `Source`, `Split`, and `Preview` variants, plus a cycle action bound to `Secondary-Shift-V`. Rendering hides the editor pane in preview mode and hides the preview pane in source mode while sharing the same active `EditorTab`, document, scroll handles, preview blocks, syntax highlighting cache, and text handle path.

The requested feature should make these modes explicit from the user's point of view: Edit, Split Preview, and Read. The current split layout remains the default and the existing view-state data flow should be reused rather than creating separate document surfaces.

Data flow:

1. User activates a menu item, toolbar control, or shortcut.
2. The GPUI action handler updates `MarkionApp.view_mode`.
3. `render` chooses which pane(s) are visible from `view_mode`.
4. Editor and preview panes keep reading from the active tab's document/caches; no document mutation or derived-state invalidation occurs solely because the mode changed.

## Goals / Non-Goals

**Goals:**

- Provide three explicit modes named Edit, Split Preview, and Read.
- Keep Split Preview as the current two-pane live-preview experience.
- Add direct shortcuts for each mode, using `secondary-` modifier conventions for Cmd on macOS and Ctrl on Windows/Linux.
- Keep mode switches local to the open window/app state and non-destructive to document, cursor, selection, undo/redo, scroll, search, and preview caches.
- Localize all new menu labels, status labels, and shortcut-reference entries.

**Non-Goals:**

- No single-surface WYSIWYG editor surface.
- No rendered-preview editing in Read mode.
- No persistence of the active mode across launches unless a later preference change explicitly adds it.
- No Markdown parser, export, or derived-cache changes.

## Decisions

1. Model modes as a small enum with user-facing names Edit, Split Preview, and Read.

   Rationale: the existing `ViewMode` enum is already the correct architectural home for this state. Implementation can either rename `Source` to `Edit` and `Preview` to `Read`, or keep internal names and expose new labels; renaming is clearer if the local blast radius stays small.

   Alternative considered: model each mode as separate layout flags. That would make invalid combinations possible and complicate menu/shortcut state.

2. Add direct set-mode actions in addition to the existing cycle action.

   Rationale: cycling is convenient but does not satisfy direct switching when the user knows the target mode. Direct actions also make menu entries and shortcut-reference tests straightforward.

   Alternative considered: keep only `ToggleViewMode`. That preserves current behavior but forces users to cycle through intermediate modes.

3. Use numbered direct shortcuts: `Secondary-Alt-1` for Edit, `Secondary-Alt-2` for Split Preview, and `Secondary-Alt-3` for Read.

   Rationale: the app already uses `Secondary-1/2/3` for heading levels, so adding Alt creates a related but non-conflicting mode switch family. The shortcuts are platform-adaptive through GPUI's `secondary-` convention.

   Alternative considered: mnemonic letter shortcuts such as `Secondary-Alt-E/S/R`. Those are easier to remember in English but less language-neutral and more likely to collide with future editor commands.

4. Reuse the current pane rendering and cache access paths.

   Rationale: hiding panes at render time already preserves tab state and avoids extra Markdown parsing. Mode switching should not call document edit APIs, clear caches, reset scroll handles, or rebuild preview data outside the normal per-version cache path.

   Alternative considered: separate read-only document model for Read mode. That would duplicate state and risk stale previews.

## Risks / Trade-offs

- Shortcut collisions on some platforms or keyboard layouts -> Mitigate by adding key-binding tests/shortcut-reference assertions and keeping the existing cycle shortcut as a fallback.
- Internal rename churn from `Source`/`Preview` to `Edit`/`Read` -> Mitigate by making the rename mechanical and covering `ViewMode::next` behavior with tests.
- Menu wording grows longer, especially in Chinese -> Mitigate by using concise localized labels and checking both native menus and in-app dropdown labels.
- Hidden panes may have stale visual scroll on return -> Mitigate by preserving the existing `EditorTab` editor/preview `ScrollHandle`s and adding a scenario/test for mode switches preserving scroll state.
