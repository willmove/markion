## 1. Single Redo Shortcut

- [x] 1.1 Change `menu_shortcuts::REDO` to the single `secondary-y` descriptor with Ctrl+Y / Cmd+Y labels, remove the extra Redo keybinding installation, and remove alias-only shortcut metadata code if it has no remaining caller.
- [x] 1.2 Update both default and extended-heading Help shortcut catalogs so the Undo/Redo row documents Ctrl+Z + Ctrl+Y on Windows/Linux and Cmd+Z + Cmd+Y on macOS.
- [x] 1.3 Update shortcut metadata, menu wiring, and localized shortcut-reference tests to prove Redo exposes exactly one active binding/label and no Ctrl/Cmd+Shift+Z combination.

## 2. Hover-Tracked In-Window Menus

- [x] 2.1 Add a guarded menu-hover state transition that switches `active_menu` only when a menu session is already open and the hovered `AppMenu` differs, notifying GPUI only for an actual change.
- [x] 2.2 Extend `menu_title_button` with the pointer listener and wire all six top-level titles to their corresponding `AppMenu`, preserving the existing click toggle, theme styling, localization, and outside-click dismissal.
- [x] 2.3 Add tests for idle hover, cross-menu switching, repeated hover on the active title, post-dismissal hover, and complete hover wiring across File, Edit, View, Format, Export, and Help.

## 3. Verification

- [x] 3.1 Run `cargo fmt --check` and `cargo test` for the root package, fixing any formatting or regression failures without changing document-version cache behavior.
- [x] 3.2 Run `cargo build` and manually smoke-test that Ctrl+Y performs Redo, the Edit/Help surfaces show only Ctrl+Y, Format-to-View pointer movement switches the open dropdown, idle hover stays closed, and outside click still dismisses the menu session.
