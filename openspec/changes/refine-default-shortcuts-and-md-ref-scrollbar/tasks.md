## 1. Shortcut Registry

- [x] 1.1 Change `INLINE_CODE` to `` secondary-shift-` `` with labels `Ctrl+Shift+\`` / `Cmd+Shift+\``
- [x] 1.2 Change `SET_VISUAL_EDIT_MODE` to `secondary-e` (`Ctrl+E` / `Cmd+E`)
- [x] 1.3 Change `SET_READ_MODE` to `secondary-r` (`Ctrl+R` / `Cmd+R`)
- [x] 1.4 Add `OPEN_FOLDER` (`open-folder`, `secondary-shift-o`, `Ctrl+Shift+O` / `Cmd+Shift+O`) to `ALL` after `OPEN_DOCUMENT`

## 2. Key Dispatch and Menus

- [x] 2.1 Bind `OpenFolder` in `bind_app_keys` via `OPEN_FOLDER`
- [x] 2.2 Pass `menu_shortcuts::OPEN_FOLDER` to the in-window File → Open Folder row
- [x] 2.3 Update registry / open-folder / refined-default tests (including GPUI parse of the backtick chord)

## 3. Catalog, Docs, Scrollbar

- [x] 3.1 Insert Open Folder into Files catalog ids/keys/labels; update Editing and View catalog keys for the new defaults
- [x] 3.2 Wire Markdown Reference body to `markdown_reference_scroll` + `pane_scrollbar_view` (`PaneScrollTarget::MarkdownReference`) so overflowing content shows the shared right-side thumb
- [x] 3.3 Update `docs/keyboard-shortcuts.md` and `docs/faq.md` for the new chords and Open Folder

## 4. Verify

- [x] 4.1 Run focused tests plus `cargo test --workspace` and `cargo fmt`
- [x] 4.2 Run `openspec validate refine-default-shortcuts-and-md-ref-scrollbar`
