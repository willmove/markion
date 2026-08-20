## 1. Actions and handlers

- [x] 1.1 In `src/app/mod.rs`, add `ReportIssue` and `OpenOnlineDocs` to the `actions!` list (after `CheckForUpdates`), and add `GITHUB_ISSUES_URL = "https://github.com/willmove/markion/issues/new"` and `GITHUB_DOCS_URL = "https://github.com/willmove/markion#readme"` beside the existing `GITHUB_REPO_URL` constant.
- [x] 1.2 In `src/app/search.rs` (next to `about`), add `report_issue` and `open_online_docs` handler methods on `MarkionApp`: set `active_menu = None`, call `cx.open_url(<constant>)`, then `cx.notify()`. No status-bar text, no shortcut registrations.

## 2. Localization

- [x] 2.1 In `src/i18n.rs`, add `ItemReportIssue` and `ItemOnlineDocs` variants to the Help-menu items block of the `Msg` enum.
- [x] 2.2 Fill the seven per-language translation arms (En, ZhHans, ZhHant, Ja, Fr, De, Es) with the labels from design.md §4 — no trailing ellipsis.

## 3. Menu surfaces

- [x] 3.1 In `src/app/root_view.rs` (`AppMenu::Help` arm of the menu panel), insert after Check for Updates: `menu_separator`, `action_item!(Msg::ItemReportIssue, report_issue, ReportIssue)`, `action_item!(Msg::ItemOnlineDocs, open_online_docs, OpenOnlineDocs)` (both without a shortcut argument), `menu_separator`, keeping About Markion last.
- [x] 3.2 In `src/app/bootstrap.rs` (`install_menus`), mirror the same items and order in the native Help `Menu` using `MenuItem::action` / `MenuItem::separator`.
- [x] 3.3 Check the longest localized label ("Informar de un problema", Es) against the Help `dropdown_width` (`px(236.)` in `src/app/mod.rs`); widen that constant only if the label would clip. Leave the per-language `menu_left` title offsets untouched.

## 4. Tests and verification

- [x] 4.1 Extend the in-window Help-menu source-scan test in `src/app/tests.rs` (~line 298) to assert the panel contains `Msg::ItemReportIssue` and `Msg::ItemOnlineDocs` and that both precede `Msg::ItemAboutMarkion`.
- [x] 4.2 Extend the native-menu bootstrap test in `src/app/tests.rs` (~line 2125) to assert `install_menus` registers both new actions.
- [x] 4.3 Run `cargo test` (root package) and fix any fallout; then manually verify on Windows: clicking each item closes the dropdown and opens the correct URL in the default browser, and switching the interface language relabels both items in the dropdown.

> All tasks are menu-chrome wiring only — none touch document state, so the cached-per-version Markdown invariants are unaffected by construction.
