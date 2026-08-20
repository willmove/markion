# Tab bar context menu

## Why

Right-clicking a workspace tab currently does nothing. The tab bar supports only left-click switch and the per-tab `×` close button, so common per-tab operations (close others, rename the file behind the tab, copy its path, reveal it in the file manager) require detouring through the menu bar or the file tree — and some (close others / close to the right) do not exist anywhere in the app. The app already has two right-click context menus (file tree, preview pane) with identical structure; extending the same interaction to the tab bar is consistent and cheap.

## What Changes

- Add a right-click context menu to tab-bar items in `tab_bar_view` (`src/app/editing.rs`), following the existing `FileTreeContextMenu` / `PreviewContextMenu` pattern (menu state struct on `MarkionApp`, `anchored()` + `occlude()` popup view, click-away dismissal, mutual exclusion with other menus).
- Menu actions, operating on the **right-clicked tab** via the established "switch to the clicked tab first, then run the action" idiom (same as the `×` button):
  - **Close Tab** — close the clicked tab (dirty confirmation unchanged).
  - **Close Others** — close every other tab; dirty tabs are **kept open by default** and reported in a summary dialog offering per-tab handling or a single "discard all N dirty tabs" confirmation.
  - **Close to the Right** — same policy as Close Others, restricted to tabs right of the clicked one.
  - **Rename…** — reuse the file-tree rename pipeline (`PendingNameInput` + `FileTree::rename_unique` + existing tab-repath-on-rename logic); disabled for untitled tabs and while the target document is dirty (existing "save before rename" rule).
  - **Copy File Path** — copy the tab's absolute path to the clipboard with status feedback; disabled for untitled tabs.
  - **Reveal in File Manager** — reuse `reveal_in_system_file_manager`; disabled for untitled tabs.
- Add middle-click-to-close on tab items as a companion interaction (currently absent app-wide).
- Add disabled-item rendering and the first menu-group separators to the shared context-menu idiom (existing menus are flat lists).
- New i18n strings for all labels/dialogs via `src/i18n.rs` (English + Simplified Chinese).

Non-goals: tab pinning, tab drag reordering, split panes, "reopen closed tab" history, and shortcut hints inside the context menu.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `workspace`: add a requirement covering the tab-bar right-click menu (actions, target semantics, dirty-tab policy for batch closes, untitled-tab disabled states, middle-click close).

## Impact

- `src/app/mod.rs` — new `TabContextMenu` state + action enum, menu mutual-exclusion updates.
- `src/app/editing.rs` — right-click/middle-click listeners in `tab_bar_view`; batch-close helpers near `close_tab`.
- `src/app/root_view.rs` — `tab_context_menu_view` render + click-away wiring; render the rename prompt line outside the file-tree panel when invoked from a tab and the tree panel is hidden.
- `src/app/workspace.rs` / `src/app/documents.rs` — action handlers (reuse rename/reveal/clipboard paths).
- `src/i18n.rs` — new `Msg` entries.
- `src/app/tests.rs` — menu open/dispatch/stale-index/dirty-policy regression tests.
- Invariants preserved: derived-state caching untouched; menu state is presentation-only; all closings go through the existing dirty-confirmation and recovery-file cleanup paths (`close_tab_confirmed`), never silent removals.
