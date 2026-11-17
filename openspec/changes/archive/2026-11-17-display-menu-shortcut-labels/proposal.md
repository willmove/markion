## Why

Markion's in-window dropdown menus currently show only action names, so users cannot discover an action's keyboard shortcut while browsing the menu even though many actions already have bindings. Showing the active platform's shortcut beside each bound menu item makes those commands easier to learn and brings the implementation into line with the existing shortcut-system contract.

## What Changes

- Add a shortcut column to the in-window File, Edit, View, Format, Export, and Help dropdowns.
- Show a platform-appropriate, user-facing shortcut label only for menu actions that have an active binding; leave unbound items without a shortcut marker.
- Keep menu action labels localized while aligning shortcut labels consistently and widening dropdowns as needed to avoid clipping.
- Keep displayed menu shortcuts synchronized with the application's binding definitions and conditional menu contents, including the configured H1–H5/H1–H6 heading depth.
- Preserve native menu behavior and all existing action handlers.
- Non-goals: add or change keybindings, make shortcuts user-configurable, add shortcut labels to context menus, or alter Markdown parsing and derived-state caches.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `markdown-editing`: Clarify that every bound action in the in-window application menus displays its current platform-specific shortcut beside the localized action label, while unbound actions do not show a shortcut.

## Impact

- Affected code: in-window menu construction and styling in `src/app/root_view.rs`, menu sizing in `src/app/mod.rs`, and shared shortcut-label/binding metadata near `src/app/bootstrap.rs` or `src/i18n.rs`.
- Tests will cover bound versus unbound menu items, platform-specific labels, conditional heading entries, and layout/source-of-truth consistency.
- No external API, persistence format, or dependency changes.
- The document-version, derived Markdown cache, syntax-highlight cache, and cached text-handle invariants are untouched.
