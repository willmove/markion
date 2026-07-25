## 1. Submenu state

- [x] 1.1 Add File-scoped Open Recent submenu open/close state on `MarkionApp`, cleared when `active_menu` leaves File or the menu closes
- [x] 1.2 Wire hover/click helpers so the Open Recent parent toggles the submenu without closing the File dropdown

## 2. File menu UI

- [x] 2.1 Replace the flat Open Recent section in `AppMenu::File` with a single parent row (`Msg::ItemOpenRecent` + submenu affordance)
- [x] 2.2 Render an adjacent occluded submenu panel listing recent paths (or empty placeholder) and Clear Recent Files; keep existing open/clear handlers
- [x] 2.3 Ensure pointer can move from parent row into the submenu without dismissing either panel; outside click closes both

## 3. Tests and verification

- [x] 3.1 Update `open_recent_menu_is_wired_in_file_dropdown` (and related structure asserts) so recent paths / Clear live under the submenu builder, not as File siblings between Open Folder and Save
- [x] 3.2 Run `cargo test` for affected app menu tests and smoke-check File → Open Recent nesting with a full recent list
