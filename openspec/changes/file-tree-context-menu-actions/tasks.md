## 1. Context Menu State and Rendering

- [x] 1.1 Add file-tree context-menu state to `MarkionApp` covering target path/kind, blank-space/workspace target, and screen position.
- [x] 1.2 Render a themed in-app context menu overlay for Files panel targets and close it after action, click-outside, or Escape.
- [x] 1.3 Add right-click handlers for file rows, folder rows, and blank/background tree space without changing left-click open/toggle behavior.

## 2. Move Existing Controls into the Context Menu

- [x] 2.1 Remove the always-visible file-tree filter input and `X` clear button from the Files panel body.
- [x] 2.2 Remove the always-visible `New`, `Dir`, `Ren`, `Del`, and `Ref` toolbar buttons from the Files panel body.
- [x] 2.3 Wire Create File, Create Folder, Rename, Delete, Refresh, and Filter Files context-menu items through existing command helpers where possible.
- [x] 2.4 Keep existing keyboard shortcuts for file-tree actions functional after the visible controls are removed.

## 3. New Context Actions

- [x] 3.1 Add file-row Open and Open in New Tab context actions.
- [x] 3.2 Add Show in System File Manager for file, folder, and workspace targets using platform-specific standard-library commands.
- [x] 3.3 Surface reveal/open failures through localized status messages without modifying editor state.

## 4. Localization and Verification

- [x] 4.1 Add `Msg` variants and English/Simplified Chinese translations for all new context-menu labels and status text.
- [x] 4.2 Add focused tests for context-menu action availability, file/folder/background target scoping, and i18n coverage.
- [x] 4.3 Run `cargo fmt`, `cargo test`, and `openspec validate file-tree-context-menu-actions`.
- [ ] 4.4 Manually verify the Files panel no longer shows the red-box controls, right-click menus appear for files/folders/background space, and existing file-tree behavior is preserved.
