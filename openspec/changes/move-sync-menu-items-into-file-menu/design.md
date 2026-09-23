## Context

Menus exist on two surfaces that must stay in lockstep: the native menu tree installed by `install_menus` in `src/app/bootstrap.rs` (File/Edit/View/Format/Export/Sync/Help) and the in-window menu bar in `src/app/root_view.rs`, driven by the `AppMenu` enum in `src/app/mod.rs`. The in-window surface renders nested submenus (File → Open Recent, Format → Images, Sync → Advanced Git Tools) as absolutely-positioned flyout panels anchored at `AppMenu::<menu>.dropdown_left(language) + dropdown_width - overlap` with hand-computed `SUBMENU_TOP` row offsets. Menu chrome is static per render — none of the versioned derived-state caches (preview blocks, outline, stats) or the editor text handle are involved, so the typing-path invariants are untouched by this change. See proposal.md for motivation.

## Goals / Non-Goals

**Goals:**

- Identical menu structure on both surfaces: Backup and Sync and Advanced Git Tools submenus inside the File menu, no top-level Sync category.
- Reuse the existing flyout pattern, submenu state flags, and localized strings so no new translation work is needed.
- Keep every action id, handler, shortcut-registry entry, and default binding untouched.

**Non-Goals:**

- Re-anchoring flyouts to measured row positions (the hand-computed `SUBMENU_TOP` convention stays).
- Changing the sync center / setup surfaces, the shortcut reference, or any Git sync behavior.

## Decisions

- **Remove the `AppMenu::Repository` variant entirely** rather than keeping an empty menu. Its title button, `toggle_repository_menu`/hover handlers, dropdown-panel match arm, and `dropdown_left`/`dropdown_width` arms all go. Alternative — keep the variant as an alias for File — was rejected because source-contract tests read the menu structure literally and a dead variant would invite drift.
- **Place the two submenu parent rows after the save group in the File dropdown** (after the `document_actions_enabled` conditional block), introduced by their own separator. This groups Save / Save As with the backup submenus and matches the native File menu insert position. Alternative — near Preferences at the bottom — was rejected because the entries are document-backup concerns, not application concerns.
- **Version History moves into the Backup and Sync submenu, directly after View Sync Status.** It is a distinct action (`ShowFileVersionHistory` opens the sync center's file-history page for the active document), not a duplicate of the sync-center entry, and it is a backup view rather than a technical Git operation — so it joins Backup and Sync instead of Advanced Git Tools. The in-window flyout receives the existing `document_actions_enabled` flag and renders the row only when a text document is active, preserving today's availability; the native menu stays ungated as before.
- **In-window flyouts follow the existing convention**: a new `backup_sync_submenu_panel` module-level function plus the existing `advanced_git_submenu_panel`, both anchored to `AppMenu::File.dropdown_left(...)`/`dropdown_width(...)` with `SUBMENU_TOP` computed for the common case (document actions enabled, no DOCX import running). Conditional rows above (DOCX cancel row, save-group replacement) shift the true row position; this drift is already tolerated by the Open Recent flyout today because flyouts open on row hover, not on exact coordinate hit.
- **Submenu state mirrors the existing flags**: add `backup_sync_submenu_open` with `open_/toggle_/close_backup_sync_submenu` methods next to `advanced_git_submenu_open`, gate both flyout renders on `active_menu == Some(AppMenu::File)`, and make the three File-menu flyouts (Open Recent, Backup and Sync, Advanced Git Tools) close each other on hover using the same pattern `file_action_item!` already applies to Open Recent.
- **Unify the open-sync-center row to `GitMsg::ViewStatus`** on both surfaces. The in-window Repository menu currently labels it `GitMsg::BackupAndSync`; inside a submenu titled Backup and Sync that repeat would read as a no-op row. The native menu already uses ViewStatus.
- **Delete `GitMsg::SyncMenu`** (variant, seven-language translations, and the distinct-title test that existed to keep it apart from `BackupAndSync`). Both new submenu titles reuse existing variants, so the 7-language completeness gate needs no new strings.
- **Retune `AppMenu::Help.dropdown_left`** after removing the Sync title button: Latin-script offsets move from 454 to ~304 px and Chinese offsets from 312 to ~222 px (Help inherits the slot the Sync button vacated). Verified during GUI smoke across both script families, per the existing hand-tuning practice.

## Risks / Trade-offs

- [Hand-tuned Help offset is wrong for some language] → GUI smoke opens the Help dropdown in English and Chinese families (the two tuned columns) and adjusts the constants before release.
- [Flyout tops drift when conditional rows differ from the common case] → accepted, same tolerance as Open Recent; flyouts open via parent-row hover so vertical misalignment does not block interaction.
- [Archived-spec contradiction with pending changes] → `simplify-sync-and-view-menu` (complete, unarchived) and `simplify-git-sync-setup-and-messages` describe the current Sync menu surfaces; archive both before this change so the synced specs never simultaneously describe a top-level Sync category and its removal.
- [Source-contract tests encode the old menu shape] → they are updated in the same change; no release ships between edit and test update.

## Migration Plan

No data or preference migration — a pure UI relocation with unchanged action ids, so customized shortcut overrides keep working. Rollback is reverting the implementing commit.
