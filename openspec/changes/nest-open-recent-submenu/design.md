## Context

`persist-session-and-recent-files` added a bounded recent-files list and exposed it in the in-window File dropdown. Implementation currently renders a muted `Open Recent` label, then flattens every recent path plus `Clear Recent Files` as siblings of New / Open / Save in the same panel (`active_menu_dropdown` → `AppMenu::File` in `src/app/root_view.rs`). With ~10 recent entries the File menu becomes taller than the window, pushing Save and later items out of reach.

The in-window menu bar today only tracks top-level `active_menu: Option<AppMenu>` (click/hover between File…Help). There is no nested-submenu state yet. Native OS menus are out of scope for this change.

## Goals / Non-Goals

**Goals:**

- Make `Open Recent` a single parent row in the File dropdown that reveals a nested submenu.
- Place recent path items, empty-state placeholder, and `Clear Recent Files` only inside that submenu.
- Keep open/clear semantics identical (`open_recent_path`, `clear_recent_files`).
- Keep the primary File panel short enough that Save through Quit remain visible without scrolling the menu itself.
- Preserve existing click-outside-to-close and top-level menu hover switching.

**Non-Goals:**

- General multi-level submenu framework for every menu.
- Changing recent-list size, persistence, or session restore.
- Native OS menu bar Open Recent entries.
- Keyboard navigation redesign beyond what is needed to open/close the submenu with pointer.

## Decisions

### 1. Add a File-scoped submenu flag, not a new `AppMenu` variant

Track something like `file_submenu: Option<FileSubmenu>` (or a bool `open_recent_submenu_open`) cleared whenever `active_menu` leaves `File` or the whole menu closes. Rationale: Open Recent is the only nested surface today; expanding `AppMenu` would over-model a one-off. Alternatives considered: encoding nested state inside `AppMenu::File` with payload (rejected: complicates hover switching across top-level titles); always-visible flyout without state (rejected: cannot dismiss cleanly / hard to theme).

### 2. Parent row is interactive; children live in an adjacent panel

Replace the muted section header with a hover/click parent item labeled `Msg::ItemOpenRecent`, with a simple affordance (e.g. trailing `›` / chevron) so it reads as a submenu. On hover (and optionally click) of that row while File is open, show a second absolutely positioned panel to the right of the File dropdown containing:

1. recent paths (most recent first), or the localized empty placeholder when empty;
2. then `Clear Recent Files`.

Choosing a child runs the existing handlers and closes menus. Hovering other File rows closes the submenu. Alternatives considered: keep flat list but scroll the File panel (rejected: does not match the requested hierarchy and still clutters the primary list); modal recent-files dialog (rejected: heavier than a submenu).

### 3. Hit-testing and outside-click

Both the File panel and the Open Recent submenu panel must `.occlude()` so clicks do not fall through (same lesson as the top-level dropdown). Outside click continues to clear `active_menu` and the submenu flag together. Moving the pointer from the parent row into the submenu must not close either panel (avoid a dead gap or use overlapping hover targets).

### 4. Spec / test contract

Delta the chrome-platform Open Recent requirement so scenarios assert submenu nesting (paths and Clear live under Open Recent, not as File siblings). Update `open_recent_menu_is_wired_in_file_dropdown` (and add a structure assertion) so the File panel embeds a submenu builder rather than inlining path buttons between Open Folder and Save.

### 5. i18n

Reuse `Msg::ItemOpenRecent`, `ItemOpenRecentEmpty`, and `ItemClearRecentFiles`. No new user-visible strings unless a chevron-only affordance needs none. Path labels remain filename-based as today.

## Risks / Trade-offs

- [Hover gap closes submenu] → Overlap parent/submenu bounds slightly or keep submenu open while pointer is over either panel; smoke-test manually.
- [Submenu clipped at window edge] → Prefer opening to the right; if near the right edge, fall back to left of the File panel (optional polish; primary platforms have room for the current File width).
- [Spec conflict with flat Open Recent wording in `persist-session-and-recent-files`] → This change’s delta supersedes flat-list presentation; archive/sync order should apply this after that change’s Open Recent requirement lands, or merge wording when archiving.
- [Only one nested menu] → Acceptable trade-off; document that the submenu flag is File/Open-Recent-specific until a second consumer appears.

## Migration Plan

No data migration. Behavior-only UI change; users see Open Recent as a nested item after upgrade. Rollback = revert the menu UI/state commit.

## Open Questions

- None blocking: hover-to-open vs click-to-toggle for the parent row — default to hover-open when File is already open (matches common desktop menus), with click also accepted to open.
