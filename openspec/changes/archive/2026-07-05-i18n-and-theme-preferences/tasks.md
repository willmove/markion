# Implementation Plan: i18n and theme preferences

## Overview

Most of this change is already implemented in the working tree (`src/i18n.rs`, model/storage changes, Preferences panel, expanded theme catalog). The tasks below first verify each already-built piece against the spec, then complete the one genuinely outstanding item: wiring the **View → Language submenu** into both native menus and the in-app dropdown (the `Msg::ItemLanguage` / `SetLanguageEn` / `SetLanguageZh` scaffolding currently has no menu surface). Each task is scoped to a single, testable commit. The cached-per-version Markdown invariants are not on this path and must remain untouched.

## Tasks

- [x] 1. i18n module (`src/i18n.rs`)
  - [x] 1.1 Verify `Language { En, Zh }` with `code()` / `from_code()` / `native_name()` / `all()` / `Default = En`, and that `from_code` accepts `zh`, `chs`, `zh-cn`, `zh-hans`, `chinese` (case-insensitive) and falls back to English for unknown/empty.
  - [x] 1.2 Verify `Msg` enum and that `t` / `tf` are exhaustive `match` over `Msg` for both languages — confirm the project does not compile if a variant lacks a translation.
  - [x] 1.3 Verify `tf` positional `{0}`/`{1}` substitution via `substitute`, including out-of-range placeholders kept verbatim.
  - [x] 1.4 Verify `shortcut_reference(lang)` returns the English/Chinese shortcut blocks and that the English block matches the pre-i18n literal (regression guard).
  - [x] 1.5 Verify `sidebar_tab_label(lang, tab)` returns localized Files/Outline labels.
  - [x] _Requirements: ui-i18n (translate through i18n layer; compile-time exhaustiveness)_

- [x] 2. i18n test coverage (`src/i18n.rs` test module)
  - [x] 2.1 Confirm the six i18n tests pass: `language_round_trips_through_its_code`, `language_from_code_accepts_common_aliases`, `english_shortcut_reference_is_unchanged`, `chinese_shortcut_reference_is_translated`, `substitute_replaces_positional_placeholders`, `tf_uses_template_per_language`.
  - [x] 2.2 Confirm the exhaustiveness guard `every_message_returns_non_empty_text_for_every_language` covers all current `Msg` variants for both languages.
  - [x] _Requirements: ui-i18n (every message returns non-empty text)_

- [x] 3. Preferences persistence (`src/model.rs`, `src/storage/preferences.rs`)
  - [x] 3.1 Verify `AppPreferences.language: String` defaults to `"en"` and that the field doc notes why it stays a raw `String` (model stays dependency-free).
  - [x] 3.2 Verify `parse_app_preferences` handles `language=…` (non-empty → store; empty/missing → keep default) and `render_app_preferences` emits `language=<code>` as the last line.
  - [x] 3.3 Verify the extended round-trip test in `src/lib.rs` covers `language=zh`, the missing-line default, and the unknown-value fallback.
  - [x] _Requirements: ui-i18n (selection + persistence + fallback); theme-preferences (single prefs file)_

- [x] 4. App state wiring (`src/main.rs`)
  - [x] 4.1 Verify `MarkionApp.language: Language` is initialized from `Language::from_code(&preferences.language)` at load and written back as `self.language.code()` on save.
  - [x] 4.2 Verify `tr()` / `trf()` helpers delegate to `t` / `tf` with the active language.
  - [x] 4.3 Verify `apply_language(lang, cx)` updates state, calls `persist_preferences()`, reinstalls native menus via `install_menus(self.language, cx)`, sets `StatusLanguageSet`, and closes any open dropdown.
  - [x] _Requirements: ui-i18n (switch takes effect immediately; persistence)_

- [x] 5. Translated UI surfaces (`src/main.rs`)
  - [x] 5.1 Verify native OS menus (`install_menus`) build every label via `t(language, Msg::…)`.
  - [x] 5.2 Verify the in-app dropdown (`active_menu_dropdown`) renders via the `action_item!` macro / `t(language, …)` and that `dropdown_left`/`dropdown_width` carry the per-language pixel arms.
  - [x] 5.3 Verify status bar, dialogs, search panel, file tree, and preferences panel all render via `t`/`tf`/`shortcut_reference`/`sidebar_tab_label` — no hard-coded user-visible English literals remain in these surfaces.
  - [x] _Requirements: ui-i18n (all UI chrome translated)_

- [x] 6. Theme catalog (`src/model.rs`, `src/main.rs`)
  - [x] 6.1 Verify `builtin_theme_definitions()` returns 14 themes with the original six first and in order (Paper, Ink, Solar, Forest, Rose, Graphite), enabling `cycle_theme` legacy behavior.
  - [x] 6.2 Verify `ThemeColors::new` is a `const fn` and the theme table reads as labelled hex values.
  - [x] 6.3 Verify the test `builtin_theme_table_exposes_popular_themes_with_unique_names` (≥11 themes, original six first, unique names) passes.
  - [x] _Requirements: theme-preferences (built-in catalog)_

- [x] 7. Preferences panel UI (`src/main.rs` `preferences_panel_view`)
  - [x] 7.1 Verify the theme swatch grid renders one card per built-in + custom `.theme`, shows a multi-segment palette preview, marks the active theme with a check and a highlight border, and that built-ins win on name collisions.
  - [x] 7.2 Verify clicking a theme card calls `apply_theme_by_name`, applies the theme immediately, and persists it.
  - [x] 7.3 Verify the language picker renders one pill per `Language::all()` labelled with `native_name()`, marks the active language, and calls `apply_language` on click.
  - [x] _Requirements: theme-preferences (theme picker by swatch; language picker); ui-i18n (selectable language)_

- [ ] 8. **NEW** View → Language submenu (`src/main.rs`)
  - [ ] 8.1 Add a Language submenu to the View arm of `install_menus` that lists each `Language::all()` entry via `t(language, Msg::ItemLanguage)` header + `lang.native_name()`, bound to `SetLanguageEn` / `SetLanguageZh` actions (and a generic path if more languages exist).
  - [ ] 8.2 Add the matching Language submenu to the View arm of `active_menu_dropdown` (in-app dropdown), reusing the same action bindings and labels.
  - [ ] 8.3 Verify selecting an entry from either surface routes through `apply_language` → state update + persist + menu retranslation + status confirmation.
  - [ ] 8.4 Add/extend a test asserting the View menu exposes a Language entry for every supported language (or assert via the dropdown builder path used by other View-menu tests).
  - [ ] _Requirements: ui-i18n (View → Language submenu)_

- [ ] 9. Final verification
  - [x] 9.1 Run `cargo test` — all existing tests plus the i18n/theme tests pass. _(99 lib + 6 main = 105 passed, 0 failed.)_
  - [ ] 9.2 Run `cargo build` clean with no warnings related to i18n/theme dead code (the previously dead `Msg::ItemLanguage` / `SetLanguageEn` / `SetLanguageZh` are now used). _Blocked on task 8: these symbols are currently unused because no menu surfaces them; `cargo build` itself is warning-clean, but the dead-code condition this task targets is only resolved once the View → Language submenu is wired._
  - [ ] 9.3 Manually verify: switch language from both Preferences panel and View → Language submenu; restart to confirm persistence; confirm document content is never translated.
  - [ ] _Requirements: all_
