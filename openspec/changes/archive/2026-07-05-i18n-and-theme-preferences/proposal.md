## Why

Markion's UI chrome was hard-coded to English and offered only six built-in themes. As the menu bar, status bar, dialogs, and preferences panel grew, every user-visible string needed editing in many places, and the small theme list did not cover common editor palettes users expect. This change introduces a dependency-free internationalization layer and expands the built-in theme catalog, both surfaced through an in-app Preferences panel — bringing the editor to parity with the user experience described in the project README.

## What Changes

- **i18n layer (`src/i18n.rs`, new):** A hand-rolled, zero-dependency internationalization module. A compile-time-checked `Msg` enum (~240 keys) covers every user-visible UI string (menus, items, status bar, dialogs, search panel, file tree, preferences). A `Language` enum (`En`, `Zh`) drives `t()` (static labels), `tf()` (positional `{0}` templates), `shortcut_reference()`, and `sidebar_tab_label()`. Adding a language is a `match`-arm extension; missing a translation site is a compile error.
- **Multilingual UI:** Every menu label (native OS menus via `install_menus` and the in-app dropdown), the status bar, dialogs, search panel, file tree, and preferences panel are driven through `t` / `tf`. Switching language retranslates native menus immediately. Interface language defaults to English; unknown/empty persisted values fall back to English.
- **Preferences persistence for language:** `AppPreferences` gains a `language: String` field (raw code, e.g. `"en"`/`"zh"`) persisted in the preferences file as `language=…`. The `model` layer stays dependency-free; the UI interprets the code via `Language::from_code`.
- **Expanded theme catalog (`src/model.rs`):** `builtin_theme_definitions()` returns 14 themes — the original six (Paper/Ink/Solar/Forest/Rose/Graphite, kept first and in order for `cycle_theme` legacy compat) plus GitHub Light/Dark, Solarized Light/Dark, One Light/Dark, and Tokyo Night/Light. `ThemeColors` gains a `const fn new` so the table reads as labelled hex values.
- **Preferences panel (`src/main.rs`):** A swatch-grid theme picker (one card per built-in + custom `.theme` file, 4-segment color preview, ✓ on active) and a language picker (one pill per `Language::all()`, labelled with `native_name()`). Selections persist across launches.
- **Tests:** `src/i18n.rs` ships a dedicated test module (round-trip, alias tolerance, regression guard for the English shortcut text, translation presence, `substitute` behavior, and an exhaustiveness guard asserting every `Msg` returns non-empty text for every language). `src/lib.rs` extends the preferences round-trip test to cover `language`.

### Open loose end (resolution at archive time)

`Msg::ItemLanguage` plus the `SetLanguageEn` / `SetLanguageZh` actions and their handlers are fully implemented, translated, and action-registered, but are **not surfaced in any menu** — language switching is currently reachable only via the Preferences panel. Task group 8 of this change proposed wiring a **View → Language submenu** into both `install_menus` and the in-app dropdown.

**Resolution at archive:** group 8 was **not** implemented before archiving. Rather than carry a description of an unimplemented feature into the baseline, the corresponding "View → Language submenu" requirement was **removed** from the `ui-i18n` delta spec before sync, so the archived baseline (`openspec/specs/ui-i18n/`) describes only what is actually implemented — language selection via the Preferences panel. The `Msg::ItemLanguage` / `SetLanguageEn` / `SetLanguageZh` symbols remain in the code as dormant scaffolding; surfacing a View → Language submenu (or removing that scaffolding) is a **future-change candidate**. Task group 8 and the dependent 9.2 / 9.3 are archived unchecked as a record of this deferral.

### Non-goals

- No new external crates — i18n stays hand-rolled (`pulldown-cmark` remains the only non-UI dependency of note).
- No translation of document content, the welcome Markdown, or user files — only UI chrome.
- No right-to-left layout or complex pluralization/ICU rules; `{0}`/`{1}` positional substitution is sufficient for the current string set.
- No per-cell theme token editing, no theme import/export UI beyond the existing custom `.theme` directory convention.
- Languages beyond English and Simplified Chinese are out of scope (the `Language` enum is extensible, but adding more is a follow-up).

## Capabilities

### New Capabilities
- `ui-i18n`: Runtime translation of all user-visible UI chrome across a fixed set of interface languages, with compile-time-checked message keys and persistence of the chosen language.
- `theme-preferences`: The set of built-in editor themes, the in-app Preferences panel for choosing a theme and an interface language, and persistence of those choices.

### Modified Capabilities
<!-- None yet: openspec/specs/ is empty prior to this change. -->

## Impact

- **Code:** `src/i18n.rs` (new, ~1300 lines), `src/main.rs` (large: state field `language`, `apply_language`, `tr`/`trf` helpers, retranslated menu/dropdown/status/dialog/search/tree/preferences rendering, `preferences_panel_view` swatch grid + language picker, `builtin_theme_table_exposes_popular_themes_with_unique_names` test), `src/model.rs` (`AppPreferences.language`, `builtin_theme_definitions()`, `ThemeColors::new`), `src/storage/preferences.rs` (`language=…` parse/render), `src/lib.rs` (re-exports + extended prefs test), `README.md` (feature bullets).
- **Invariants touched:** None of the cached-per-version Markdown invariants are affected — i18n is purely a UI-chrome concern and `builtin_theme_definitions()` is a static table. The `cycle_theme` legacy path is preserved by keeping the original six themes first and in order.
- **Persistence format:** Additive — a new `language=…` line in the preferences file. Older builds ignore it; this build tolerates its absence (defaults to `"en"`).
- **APIs / dependencies:** No new dependencies. `Language`, `Msg`, `t`, `tf`, `shortcut_reference`, `sidebar_tab_label`, and `builtin_theme_definitions` become part of the crate's public re-exports.
