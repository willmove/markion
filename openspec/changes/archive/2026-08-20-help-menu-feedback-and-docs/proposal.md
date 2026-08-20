# Proposal: help-menu-feedback-and-docs

## Why

The Help menu currently offers only "Check for Updates…" and "About Markion". Users who hit a problem or want to learn the app's features have no in-app path to the issue tracker or the documentation — they must already know the repository URL. Every desktop app in this space exposes "Report an Issue" and "Online Documentation" from Help; Markion's canonical documentation (README, `docs/`) and issue tracker already exist on GitHub, so linking them from the menu is cheap and closes a real discoverability gap.

## What Changes

- Add two items to the Help menu, in both surfaces that render it (the in-window dropdown used on Windows/Linux and the native menu bar installed on macOS):
  - **Report an Issue** — opens `https://github.com/willmove/markion/issues/new` in the system browser.
  - **Online Documentation** — opens `https://github.com/willmove/markion#readme` in the system browser.
- Menu order becomes: Check for Updates… · separator · Report an Issue · Online Documentation · separator · About Markion (web links grouped between the update check and the About dialog).
- Both items are pointer-driven actions with no keyboard shortcuts, and both dismiss the open menu when invoked.
- Opening uses GPUI's `App::open_url`, which hands the URL to the OS default browser; the app never renders web content itself.
- Both labels are routed through `src/i18n.rs` with translations for all seven supported languages (En, ZhHans, ZhHant, Ja, Fr, De, Es).

**Non-goals:** no in-app documentation viewer or embedded web view; no pre-filled issue body or issue-template picker; no new keyboard shortcuts or shortcut-reference entries; no change to the About dialog or update check; no telemetry.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `chrome-platform`: adds a requirement covering Help-menu external links — the two new items SHALL open fixed project URLs in the system browser via the platform shell (not an embedded view), SHALL appear in both the in-window menu bar and the native macOS menu, and SHALL be localized through the i18n layer like every other menu item.

## Impact

- **Code:** `src/app/mod.rs` (two new `actions!` entries, new URL constants beside `GITHUB_REPO_URL`, possible Help dropdown width re-tune), `src/app/bootstrap.rs` (native Help menu items), `src/app/root_view.rs` (in-window Help dropdown items), one `MarkionApp` handler method file for the two handlers (co-located with the existing `about` handler in `src/app/search.rs`), `src/i18n.rs` (two new `Msg` variants plus seven translation-table arms, and a status message if one is added).
- **Specs:** `chrome-platform` gains one requirement with scenarios; no other capability's behavior changes (the `ui-i18n` localization requirement already generically covers new menu-item strings).
- **Dependencies:** none new — `open_url` already ships in the pinned GPUI 0.2.2.
- **Invariants:** untouched — this is menu chrome only; it performs no Markdown parsing and does not interact with per-version derived-state caching, memoized highlighting, or the cached text handle. Menu action handlers run on click, never per keystroke.
- **Tests:** existing source-scanning menu tests in `src/app/tests.rs` (Help menu panel and native-menu bootstrap assertions) are extended to pin the new items; i18n coverage tests (if any key-completeness test exists) pick up the new `Msg` variants automatically.
