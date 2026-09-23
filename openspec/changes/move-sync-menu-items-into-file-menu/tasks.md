## 1. Native menu bar (bootstrap.rs)

- [x] 1.1 In `install_menus`, insert into the File menu after the Save As row a separator and two `MenuItem::submenu` entries — "Backup and Sync" (`GitMsg::BackupAndSync`: `ViewStatus`→`ShowGitSync`, `GitMsg::VersionHistory`→`ShowFileVersionHistory`, `ItemGitSyncNow`→`SyncNow`, `ItemGitResolveConflict`→`ResolveGitConflict`, `ItemGitSyncSetup`→`SetupGitSync`) and "Advanced Git Tools" (`GitMsg::AdvancedGitTools`: Commit/Fetch/Pull/Push with the existing actions) — remove the standalone Version History row from the File items, and delete the top-level Sync `Menu` block; verify with `cargo build`.

## 2. In-window menu bar (mod.rs / root_view.rs)

- [x] 2.1 Remove the `AppMenu::Repository` variant and its `dropdown_left` / `dropdown_width` arms, retune `AppMenu::Help` offsets to the vacated slot (Latin-script ~304 px, Chinese ~222 px), and drop `toggle_repository_menu` plus the Repository hover wiring; verify with `cargo build`.
- [x] 2.2 Add `backup_sync_submenu_open` state with `open_` / `toggle_` / `close_backup_sync_submenu` methods mirroring the advanced-git flag, and extend every site that resets `advanced_git_submenu_open` on menu dismissal to also reset the new flag; verify with `cargo build`.
- [x] 2.3 In the menu-bar row, remove the Sync title button; in the File dropdown arm, after the save group (Version History row removed from the `document_actions_enabled` block) add a separator and the two `menu_submenu_parent_button` rows (Backup and Sync, Advanced Git Tools), relabel the open-sync-center row to `GitMsg::ViewStatus`, and make the three File flyouts (Open Recent, Backup and Sync, Advanced Git Tools) close each other on hover via the existing `file_action_item!` pattern; verify with `cargo build`.
- [x] 2.4 Add the `backup_sync_submenu_panel` flyout (View Sync Status with its shortcut label, a `GitMsg::VersionHistory`→`ShowFileVersionHistory` row rendered only when the passed `document_actions_enabled` flag is set, Sync Now, Resolve Conflicts, Sync Setup) and re-anchor `advanced_git_submenu_panel` plus its render gate to `AppMenu::File.dropdown_left` / `dropdown_width` with `SUBMENU_TOP` constants recomputed for the new row positions; verify with `cargo build`.

## 3. Localization (i18n/git.rs)

- [x] 3.1 Delete the `GitMsg::SyncMenu` variant with its seven-language translations, reshape the distinct-menu-title test that referenced it, and update the exhaustive variant list; verify with `cargo test` that the 7-language completeness tests still pass.

## 4. Source-contract tests (app/tests.rs)

- [x] 4.1 Update the in-window/native menu-structure test: the File menu contains the Backup and Sync and Advanced Git Tools submenu tokens, no top-level Sync menu exists on either surface, and the menu-bar ordering assertion reflects File → Help without the sync category.
- [x] 4.2 Update `backup_sync_menus_group_technical_operations_and_expose_version_history` to assert the two submenus live inside the File menu on both surfaces and Version History lives inside the Backup and Sync submenu (hidden in-window when no text document is active) instead of as a top-level File row.

## 5. Verification

- [x] 5.1 Run `cargo fmt`, `cargo test`, and `cargo test --workspace`; all green with no warnings introduced.
- [ ] 5.2 GUI smoke: menu bar shows File / Edit / View / Format / Export / Help; the File menu opens both submenus with hover-open flyouts that close each other and dismiss on outside click; Version History appears inside the Backup and Sync submenu and opens the active document's file-history page (and is absent when no text document is active in-window); the Help dropdown opens at its corrected offset in an English and a Chinese language; invoking Sync Now from the submenu still runs synchronization.
- [x] 5.3 `openspec validate move-sync-menu-items-into-file-menu` passes; confirm the archive-order note (archive `simplify-sync-and-view-menu` and `simplify-git-sync-setup-and-messages` before this change) is honored when archiving.
