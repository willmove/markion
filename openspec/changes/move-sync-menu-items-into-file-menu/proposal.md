## Why

The top-level "Sync" menu holds a single narrow topic (Backup and Sync plus its advanced Git tools) yet consumes a whole menu-bar slot on both menu surfaces, while the File menu is the natural home for backup and versioning of documents. Folding its entries into the File menu as two submenus shortens the menu bar and groups document-backup concerns together without changing any behavior behind the entries.

## What Changes

- Remove the top-level "Sync" menu category from both menu surfaces — the native menu bar installed in `src/app/bootstrap.rs` and the in-window menu bar in `src/app/root_view.rs`. The menu-bar order becomes File, Edit, View, Format, Export, Help (matching the existing chrome-platform wording).
- In the File menu of both surfaces, add two submenus after the save group:
  - "Backup and Sync" (reuses the existing `GitMsg::BackupAndSync` string): View Sync Status (opens the sync center), Version History (opens the active document's file-history page), Sync Now, Resolve Conflicts, Sync Setup.
  - "Advanced Git Tools" (reuses the existing `GitMsg::AdvancedGitTools` string): Commit, Fetch, Pull, Push.
- Move the Version History row out of the File menu top level into the "Backup and Sync" submenu, after View Sync Status, on both surfaces. It is a distinct backup-view action (`ShowFileVersionHistory` opens the sync center's file-history page for the active document), not a duplicate of any planned submenu entry, so it is grouped rather than deleted; the in-window row keeps its existing availability gating on an active text document.
- Unify the open-sync-center row label across both surfaces to the existing `GitMsg::ViewStatus` ("View Sync Status") string, so the submenu's first item does not repeat the submenu title (the in-window Repository menu currently labels that row "Backup and Sync").
- Remove the now-unused `GitMsg::SyncMenu` variant and its seven-language translations, and retune the in-window Help dropdown offset now that the Sync title button no longer precedes Help.
- **BREAKING UI:** the top-level Sync menu no longer exists; its entries move under File. No action ids, action handlers, shortcut registry entries, or default bindings change — only where the entries are surfaced.

Non-goals: changing Git synchronization behavior, GPUI actions, or shortcut bindings; changing the sidebar sync center, setup dialogs, or status feedback; changing the keyboard-shortcut reference (its functional categories do not mirror menu-bar structure).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `chrome-platform`: Add a requirement placing the Backup and Sync and Advanced Git Tools submenus inside the File menu on both menu surfaces, with no top-level synchronization category remaining.

## Impact

- Affected application code: `src/app/bootstrap.rs` (native menu tree), `src/app/root_view.rs` (in-window menu bar buttons, File dropdown rows, the two flyout submenu panels and their anchoring), `src/app/mod.rs` (`AppMenu` enum and hand-tuned dropdown offsets, submenu open/close state), and source-contract tests in `src/app/tests.rs`.
- Affected localization code: `src/i18n/git.rs` — delete the `SyncMenu` variant with its seven-language translations and adjust the i18n tests that reference it; the two new submenu titles reuse existing strings, so no new translations are needed and the 7-language completeness gate keeps holding.
- In-window flyout positioning follows the existing hand-computed `SUBMENU_TOP` convention used by Open Recent and Advanced Git Tools today; the constants move to the File dropdown and the Help `dropdown_left` offsets shrink by the removed Sync button.
- Archive-order dependency: the completed-but-unarchived `simplify-sync-and-view-menu` change adds requirements describing the top-level "Sync" category title to `chrome-platform` and `ui-i18n`. Archive that change (and `simplify-git-sync-setup-and-messages`) before this one so this change's removal supersedes those scenarios cleanly instead of leaving contradictory archived specs.
- No document data, Markdown caches, preferences, shortcut overrides, dependencies, or workspace members are affected.
