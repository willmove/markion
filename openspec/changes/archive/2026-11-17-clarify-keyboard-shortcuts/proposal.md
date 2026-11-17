## Why

The Help -> Keyboard Shortcuts reference currently lists generic `Secondary-*` bindings, which obscures the actual keys users press on Windows and macOS. The sidebar toggle also uses `Secondary-Alt-B`, a three-modifier shortcut that is harder to reach than the rest of the common View shortcuts.

## What Changes

- Replace the keyboard shortcut reference body with a table-oriented layout that shows actions alongside separate Windows/Linux and macOS key columns.
- Update the displayed shortcut terminology from GPUI-internal `Secondary-*` names to explicit platform keys (`Ctrl` on Windows/Linux and `Cmd` on macOS).
- Change the toggle-sidebar binding from `Secondary-Alt-B` to `Secondary-Shift-B`, yielding Ctrl+Shift+B on Windows/Linux and Cmd+Shift+B on macOS.
- Keep existing shortcut actions, localization, and Help menu entry behavior intact.
- Non-goals: add user-configurable keybindings, change markdown formatting shortcuts, or alter derived Markdown caching behavior.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `ui-i18n`: The localized keyboard shortcut reference must present shortcut information in a platform-specific table form.
- `chrome-platform`: The application chrome must expose a clearer sidebar toggle shortcut.

## Impact

- Affected code: `src/i18n.rs`, `src/app/bootstrap.rs`, and shortcut-reference unit tests.
- No API or dependency changes.
- The typing-path cache invariants are not touched; this change is limited to static shortcut binding/Help text.
