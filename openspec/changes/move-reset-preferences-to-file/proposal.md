## Why

The reset-preferences command is currently grouped under Help, while users naturally look for preference-management actions next to File -> Preferences. Moving the reset action directly under Preferences keeps the command discoverable without changing reset behavior.

## What Changes

- Move the **Reset Preferences** menu item out of Help and place it in the File menu immediately below **Preferences**.
- Keep the existing reset confirmation dialog, status messages, action type, shortcut wiring, and localization unchanged.
- Leave the Help menu focused on help/reference and preferences-summary actions.
- Non-goals: changing preference persistence, reset semantics, keyboard shortcuts, Preferences panel layout, or derived Markdown/cache behavior.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `chrome-platform`: The menu placement contract for preferences reset changes from Help to File, immediately after Preferences.

## Impact

- `src/main.rs` - in-window and native menu builders reorder the existing reset action.
- Existing `src/i18n.rs` labels remain valid; no new UI strings are needed.
- No storage, parsing, preview, or derived-state cache changes.
