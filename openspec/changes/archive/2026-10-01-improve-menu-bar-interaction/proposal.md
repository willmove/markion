## Why

Redo currently has two active shortcuts and shows both in the Edit menu, even though Markion only needs one. The in-window menu bar also requires a second click when the pointer moves between top-level menus after opening a dropdown, which differs from established desktop menu behavior and makes menu browsing feel unnecessarily slow.

## What Changes

- Make `secondary-y` the sole Redo binding, displayed as Ctrl+Y on Windows/Linux and Cmd+Y on macOS; remove the existing `secondary-shift-z` Redo binding and its displayed shortcut.
- Keep the Edit -> Redo menu entry, native menu action, Help shortcut reference, and in-window shortcut label synchronized with the single binding.
- When an in-window dropdown is already open, moving the pointer over a different top-level menu title automatically switches the active dropdown to that menu.
- Preserve click-to-open/click-to-close behavior, click-outside dismissal, menu item actions, localization, theme styling, and normal hover styling when no dropdown is open.
- Non-goals: user-configurable keybindings, changes to undo/redo history semantics, opening menus from hover when the menu bar is idle, native OS menu interaction changes, or changes to Markdown parsing and derived-state caches.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `markdown-editing`: Redo has one platform-mapped shortcut (`secondary-y`) and every shortcut surface documents only that active binding.
- `chrome-platform`: An open in-window menu session follows pointer movement across top-level menu titles and switches the visible dropdown without another click.

## Impact

- Affected code: shared menu shortcut metadata and tests in `src/app/mod.rs` / `src/app/tests.rs`, keybinding installation in `src/app/bootstrap.rs`, localized shortcut-reference data in `src/i18n.rs`, and in-window menu title event/state handling in `src/app/root_view.rs` / `src/app/editing.rs`.
- No public API, persistence-format, or dependency changes are expected.
- Undo/redo document behavior remains unchanged; the document-version derived-state cache, syntax-highlight memoization, and cached text-handle invariants are not touched.
