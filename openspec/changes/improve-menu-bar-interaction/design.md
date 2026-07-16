## Context

Markion's six top-level in-window menu titles are rendered in `src/app/root_view.rs`. Each title currently receives only an `on_mouse_up` listener, and the selected dropdown is represented by `MarkionApp.active_menu: Option<AppMenu>`. `toggle_menu` opens or closes that state, while the main content's mouse-down handler dismisses it. Because no title handles pointer movement, an open menu session cannot transfer from one title to another without another click.

Redo uses the shared `menu_shortcuts::REDO` descriptor. Its primary binding is currently `secondary-shift-z` and `secondary-y` is stored as an alias; `src/app/bootstrap.rs` installs both, and the in-window Edit menu joins both display labels. The Help shortcut catalog in `src/i18n.rs` separately lists the Undo/Redo key pair. The `Redo` action and `EditorTab::apply_redo` history behavior do not need to change.

This is application-chrome state only. Pointer-driven menu switching flows through `active_menu` and a re-render; it does not mutate a document version or touch preview, outline, statistics, syntax-highlight, undo snapshot, or cached text-handle state.

## Goals / Non-Goals

**Goals:**

- Make `secondary-y` the only installed Redo binding and show exactly that binding on every shortcut surface.
- Switch directly between top-level in-window dropdowns when the pointer enters another title during an active menu session.
- Preserve the existing click toggle, outside-click dismissal, menu actions, localization, and theme behavior.
- Make the state transition testable without requiring a full desktop interaction harness.

**Non-Goals:**

- Change undo/redo stack semantics or the `Redo` action handler.
- Open a menu solely because the pointer crosses the idle menu bar.
- Change native OS menu tracking, add keyboard traversal, or add configurable keymaps.
- Change document rendering, parsing, or derived-state caching.

## Decisions

1. Use `secondary-y` as Redo's sole shared descriptor binding.

   `menu_shortcuts::REDO` will use the normal single-binding constructor with `secondary-y`, `Ctrl+Y`, and `Cmd+Y`. `bootstrap.rs` will install that descriptor once and remove the explicit alias installation. The in-window Edit menu will continue to read its label from the same descriptor, so it cannot display Ctrl+Shift+Z after the binding is removed. The Help shortcut catalog's default and extended-heading tables will list Undo as Ctrl/Cmd+Z and Redo as Ctrl/Cmd+Y.

   Keeping `secondary-shift-z` as a hidden binding was rejected because the requested contract is one Redo shortcut, not merely one displayed label. Hard-coding `ctrl-y` was also rejected because Markion's shortcut system uses `secondary` to preserve Ctrl on Windows/Linux and Cmd on macOS.

2. Treat hover switching as a guarded `active_menu` transition.

   Add a small state helper/method with this behavior:

   ```text
   active_menu == None                  -> remain None
   active_menu == Some(hovered_menu)    -> unchanged
   active_menu == Some(other_menu)      -> Some(hovered_menu)
   ```

   Each top-level title will send its `AppMenu` value from a pointer-move/enter listener to that helper. The helper calls `cx.notify()` only when the active menu actually changes, avoiding redundant renders while the pointer continues moving within the same title. The existing click listener remains authoritative for opening and closing a menu.

   Changing dropdowns on every title hover was rejected because it would make menus appear during ordinary pointer travel. Introducing a separate `menu_tracking` flag was rejected because `active_menu.is_some()` already exactly represents an active menu session.

3. Extend the shared top-level menu-title primitive with both click and hover callbacks.

   `menu_title_button` will keep styling and click behavior intact while accepting the pointer listener needed by every title. Centralizing the wiring avoids six slightly different event implementations. The dropdown continues to be derived from `active_menu`, so switching titles automatically moves and rebuilds the panel using the target menu's existing position, width, localized items, and theme palette.

   Handling pointer coordinates on the entire menu-bar container was considered, but it would duplicate the language-dependent title geometry and make hit testing fragile. Title-local events reuse GPUI's existing element hit testing.

4. Verify binding metadata, state transitions, and render wiring separately.

   Unit tests will assert that Redo exposes only `secondary-y` with one platform label, that bootstrap installs no second Redo key, and that the Help reference contains Ctrl/Cmd+Y rather than Shift+Z for Redo. Pure state tests will cover idle hover, switching between distinct menus, and hovering the already active title. Source/wiring tests will ensure all six title buttons supply the hover transition.

## Risks / Trade-offs

- [Risk] A mouse-move listener can fire repeatedly while the pointer remains inside a title. -> Mitigation: notify only when `active_menu` changes.
- [Risk] Adding a hover listener could interfere with click-to-close behavior on the currently active title. -> Mitigation: the hover helper is a no-op for that title, and the existing click handler still toggles it closed.
- [Risk] Shortcut labels can drift between shared menu metadata and the separate localized Help catalog. -> Mitigation: update both default and extended shortcut tables and add assertions for the installed binding and displayed keys.
- [Trade-off] Cmd+Y is less conventional than Cmd+Shift+Z on macOS. -> The explicit one-shortcut requirement is applied through Markion's existing platform-mapped `secondary-y` convention for consistency across supported platforms.

## Migration Plan

No data migration is required. Apply the shortcut metadata/catalog updates and menu hover wiring together, run formatting and tests, then build the root package. Rollback consists of restoring the prior Redo descriptor/alias binding and removing the hover listener/helper; no persisted user data is affected.

## Open Questions

None.
