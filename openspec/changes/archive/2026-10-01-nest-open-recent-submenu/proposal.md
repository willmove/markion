## Why

The in-window File menu currently inlines the full recent-files list (and Clear Recent Files) under a muted "Open Recent" label. With a bounded but non-trivial recent history, that flattens the File dropdown so tall that Save, Save As, tab actions, Preferences, and Quit scroll off-screen or become hard to reach. Nesting those entries under an Open Recent submenu restores a compact primary File menu while keeping recent-file access one hover/click away.

## What Changes

- Change the in-window File menu so **Open Recent** is a parent item that opens a **submenu** (next level), not a section header with siblings in the same panel.
- Move the recent-file path entries, the empty-state placeholder, and **Clear Recent Files** into that submenu.
- Keep open/clear behavior unchanged: choosing a path still uses the existing open-recent flow; Clear still clears the store.
- Update chrome-platform requirements (and source-structure tests) so the submenu nesting is the specified UX contract.

**Non-goals:** Changing recent-list capacity, persistence, native OS menus, other File menu items, or adding a general-purpose submenu framework beyond what Open Recent needs.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `chrome-platform`: Clarify that Open Recent is a File-menu submenu whose children are the recent paths (or empty placeholder) plus Clear Recent Files, so the primary File dropdown stays compact.

## Impact

- In-window File dropdown construction in `src/app/root_view.rs` (`active_menu_dropdown` / `AppMenu::File`).
- Likely small menu-interaction state in `src/app/` (hover/open submenu under File).
- Possibly new/adjusted i18n only if a submenu affordance needs a distinct label (prefer reusing `Msg::ItemOpenRecent`).
- Tests in `src/app/tests.rs` that assert File-menu wiring and item order around Open Recent.
- Spec delta under `chrome-platform` (aligns with / supersedes flat-list wording from `persist-session-and-recent-files`).
