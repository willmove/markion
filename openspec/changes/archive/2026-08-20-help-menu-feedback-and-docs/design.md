## Context

The Help menu is rendered by two parallel surfaces that must stay in sync:

- the in-window menu bar dropdown (Windows/Linux) — `AppMenu::Help` arm in `src/app/root_view.rs` (`menu_panel_contents`, around line 3398), items built with the `action_item!` macro that binds a `Msg` label to a `MarkionApp` handler method and a GPUI action;
- the native OS menu bar (macOS) — `install_menus` in `src/app/bootstrap.rs` (around line 170).

Actions are declared once in the `actions!` list in `src/app/mod.rs` (~line 150); handlers are `pub(super)` methods on `MarkionApp` spread across `src/app/*.rs` by concern (`check_for_updates` in `update.rs`, `about` in `search.rs`). Every user-visible string goes through `Msg` variants in `src/i18n.rs` with per-language `match` arms (En, ZhHans, ZhHant, Ja, Fr, De, Es). `GITHUB_REPO_URL` already lives in `src/app/mod.rs` (~line 168) beside other Help-menu state.

This change touches only transient menu state (`active_menu`) — it performs no document or Markdown work, so none of the derived-state caching, memoized-highlighting, or cached-text-handle invariants are involved. Data flow is: menu click → GPUI dispatches the action → handler sets `active_menu = None`, calls `cx.open_url(const)`, `cx.notify()`.

## Goals / Non-Goals

**Goals:**

- Wire two external-link items into both Help-menu surfaces with the existing action/i18n patterns, so future menu items follow exactly the same recipe.
- Open URLs through the platform shell with zero new dependencies.

**Non-Goals** (design-level, beyond the proposal's non-goals):

- No retry/fallback strategy if the OS has no default browser — GPUI's `open_url` returns `()`, so failure is not observable from the app.
- No status-bar message for these two actions (see Decisions).

## Decisions

**1. Open URLs via GPUI's `App::open_url`.**
The pinned GPUI 0.2.2 already ships `open_url(&self, url: &str)` (`gpui::App`, registry `src/app.rs:1078`), which routes to `ShellExecuteW` on Windows, `NSWorkspace` on macOS, and `xdg-open` on Linux — exactly the per-platform dispatch we'd otherwise hand-roll. Handler context (`&mut Context<MarkionApp>`) derefs to `App`, so the call site is just `cx.open_url(GITHUB_ISSUES_URL)`. Alternatives rejected: spawning `xdg-open`/`cmd /c start` via `std::process` (duplicates GPUI's platform code, adds Windows GUI-subsystem pitfalls); the `webbrowser` crate (new dependency, against the project's minimal-dependency style; `src/i18n.rs` documents the same stance for its zero-crate i18n).

**2. Two new actions: `ReportIssue` and `OpenOnlineDocs`.**
Added to the `actions!` list after `CheckForUpdates`. Handlers co-locate with the existing Help-menu `about` handler in `src/app/search.rs` (that file is the misc Help/dialog bucket) and follow the established handler shape: set `active_menu = None`, open URL, `cx.notify()`. Unlike `about`, they set no status-bar text — opening a browser is instantaneous and its effect is visible outside the app, so a status line would be noise; skipping it also avoids two more `Msg` translation surfaces. URL constants (`GITHUB_ISSUES_URL = "https://github.com/willmove/markion/issues/new"`, `GITHUB_DOCS_URL = "https://github.com/willmove/markion#readme"`) sit next to `GITHUB_REPO_URL` in `src/app/mod.rs` so all repo URLs stay in one place. If a dedicated docs site launches later, only the constant changes.

**3. Menu order: Check for Updates… · — · Report an Issue · Online Documentation · — · About Markion.**
Groups the two web links between the update check and About, keeping About last per macOS convention. In `root_view.rs` this is `action_item!(CheckForUpdates)`, `menu_separator`, the two new `action_item!`s (no shortcut argument — pointer-driven), `menu_separator`, `action_item!(AboutMarkion)`. `bootstrap.rs` mirrors the same order with `MenuItem::action` / `MenuItem::separator`. No shortcut-reference or shortcut-catalog entries are added anywhere.

**4. i18n: `Msg::ItemReportIssue` and `Msg::ItemOnlineDocs`.**
Added to the Help-menu items block in `src/i18n.rs` (~line 192) with seven translation arms each. Labels (no trailing ellipsis — the convention reserves `…` for actions that open an in-app dialog):

| Language | Report an Issue | Online Documentation |
|---|---|---|
| En | Report an Issue | Online Documentation |
| ZhHans | 反馈问题 | 在线文档 |
| ZhHant | 回報問題 | 線上文件 |
| Ja | 問題を報告 | オンラインドキュメント |
| Fr | Signaler un problème | Documentation en ligne |
| De | Problem melden | Online-Dokumentation |
| Es | Informar de un problema | Documentación en línea |

**5. Dropdown width: verify, widen only if clipped.**
`dropdown_width` for Help is currently `px(236.)` (language-independent). The longest new label ("Informar de un problema", Es) must be measured against it during implementation; if it clips, bump the Help width — the `menu_left` per-language offsets key off menu *titles*, not item widths, so widening the dropdown does not move any title.

**6. Tests follow the established source-scanning pattern.**
`src/app/tests.rs` already pins the Help dropdown contents by scanning the `AppMenu::Help =>` match arm (~line 298) and the native menu bootstrap source (~line 2125). Extend both to assert the new `Msg` variants appear, and that they sit before `ItemAboutMarkion`. Browser launching itself is not unit-testable headlessly and stays covered by the spec scenarios' manual verification.

## Risks / Trade-offs

- [No default browser / broken URL association → `open_url` silently no-ops] → Accepted for v1: failure is an OS-level condition GPUI does not surface; the About dialog already prints the repo URL as a manual fallback.
- [Longest localized label clips the Help dropdown] → Width check is an explicit task; the fix is a single constant.
- [Two menu surfaces drift (in-window vs native)] → Both are covered by the source-scanning tests, which is how the existing menu items stay pinned.
- [`#readme` anchor vs a future dedicated docs site] → Centralized constant; a future site swap is a one-line change (and, if the canonical URL changes, a follow-up spec delta).

## Migration Plan

Purely additive UI wiring with no persisted-state, preference, or file-format impact. Rollback is reverting the single commit; no cleanup or data migration is needed.
