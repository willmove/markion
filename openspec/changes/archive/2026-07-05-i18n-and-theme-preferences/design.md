## Context

Markion's UI chrome was hard-coded English with a small theme set. This change adds a dependency-free i18n layer (`src/i18n.rs`) and expands the theme catalog (`builtin_theme_definitions()`), both surfaced through the existing Preferences panel overlay in `src/main.rs`. The implementation is already present in the working tree; this design documents the decisions so the spec and tasks are grounded, and so the one open loose end (View → Language submenu) is resolved deliberately rather than left as dead scaffolding.

Constraints carried from the project context:
- Single crate, no workspace. `pulldown-cmark` is the only notable non-UI dependency; the project prefers minimal dependencies.
- The cached-per-version Markdown invariants in `src/model.rs` / `src/lib.rs` must be preserved.

## Goals / Non-Goals

**Goals:**
- Translate 100% of UI chrome via compile-time-checked message keys, so missing a translation site is a build error.
- Keep the `model` layer dependency-free: language is stored as a raw `String` code and interpreted by the UI layer.
- Ship a larger built-in theme catalog without breaking the legacy `cycle_theme` ordering.
- Make language switching reachable from both the Preferences panel and a View → Language submenu.

**Non-Goals:**
- No new crates. No ICU/pluralization, no RTL, no document-content translation.
- No theme token editor, no import/export UI beyond the existing `.theme` directory convention.
- No re-architecture of the cached Markdown pipeline — i18n is strictly UI chrome.

## Decisions

### Decision 1: Hand-rolled enum + `match` instead of a localization crate (fluent / rust-i18n)
A `Msg` enum with one variant per string, plus exhaustive `en(msg)` / `zh(msg)` `match` arms, makes a missing translation a **compile error** — stronger than runtime key lookup, which silently falls back. The cost is a hand-maintained exhaustiveness test (`every_message_returns_non_empty_text_for_every_language`), but that is a secondary guard, not the primary correctness mechanism. Adding a language is "add a variant arm in 4 functions"; adding a string is "add a variant + 2 arms".
- **Alternative considered:** `fluent` with `.ftl` resource files. Rejected: adds a dependency and a runtime loader, and loses compile-time exhaustiveness over keys.

### Decision 2: `t()` returns `&'static str`; `tf()` returns owned `String`
Static labels (`Msg::MenuFile` etc.) are the majority and can be zero-allocation `&'static str` lookups. The smaller set of templated messages (counts, paths) go through `tf` → `substitute()`, which allocates. This keeps the common render path allocation-free and bounds `tf` to where it is actually needed.
- **Alternative considered:** a single function returning `Cow<'static, str>`. Rejected: spreads `into_owned`/borrowed reasoning across every call site for little gain at this scale.

### Decision 3: `Language` lives in app state; `AppPreferences.language` is a raw `String`
`MarkionApp` holds `language: Language` (typed) for rendering. Persistence stores the lowercase code as a raw `String` in `AppPreferences`, so `model` does not depend on the `i18n` module. The UI layer interprets the code via `Language::from_code` on load and writes `Language::code` on save. This mirrors how `sidebar_tab` already tolerates forward-compat values (unknown → default).

### Decision 4: Built-in themes as a `Vec<ThemeDefinition>` from `builtin_theme_definitions()`, original six first
A table function (vs. ad-hoc `AppTheme` variants) lets the Preferences panel render built-ins and custom `.theme` files in one uniform list. `ThemeColors::new` (a `const fn`) makes the table read as labelled hex values. The comment at the call site records the ordering invariant: **the original six must stay first and in order** so `cycle_theme` and its test keep passing.

### Decision 5: Wire the View → Language submenu rather than remove the scaffolding
`Msg::ItemLanguage`, `SetLanguageEn`/`SetLanguageZh`, the handlers, and `apply_language` already exist and are translated; only the menu entry is missing. Wiring it into both `install_menus` (native) and the View arm of `active_menu_dropdown` (in-app) is a small, localized render change that reuses `Language::all()` + `native_name()` and is strictly additive. Removing the scaffolding would discard already-translated strings and reduce discoverability. → Tasks choose the wire-up path.

## Risks / Trade-offs

- **[Risk] Exhaustiveness test is hand-maintained.** The `every_message_…` test enumerates variants explicitly; a new variant could be added to `Msg` and to both `match` arms (so it compiles) but forgotten in the exhaustiveness test. → *Mitigation:* the primary guard is the compiler's exhaustive `match`, not this test; the test is a non-empty-text backstop, not the correctness mechanism. A future task could derive the list via a macro to remove the drift surface.
- **[Risk] Per-language pixel tuning for the dropdown.** `dropdown_left`/`dropdown_width` carry hand-tuned offsets per language for CJK width. Adding a language means hand-tuning a new arm. → *Mitigation:* documented at the call site; accepted as the cost of pixel-precise menus without a layout engine.
- **[Risk] Theme name is the persistence key.** Renaming a built-in orphans saved selections. → *Mitigation:* spec states names are identity keys and renames SHALL be avoided; the ordering invariant for the legacy six is captured in code comments and a test.
- **[Trade-off] `model` carries language as an opaque `String`.** Slightly less type safety at the persistence boundary. Accepted to keep `model` dependency-free, matching the existing `sidebar_tab` forward-compat pattern.

## Data flow (UI chrome path — does not touch the cached Markdown pipeline)

```
load → AppPreferences.language (String, e.g. "zh")
        → Language::from_code → MarkionApp.language (typed Language)

render → t(MarkionApp.language, Msg::…)           // menus, dropdown, status, dialogs, panels
        → tf(language, msg, &[args])            // dynamic status / counts
        → shortcut_reference(language)          // Help shortcut reference
        → sidebar_tab_label(language, tab)      // sidebar tab label

switch language (Preferences pill OR View → Language)
        → apply_language(lang)
            → self.language = lang
            → persist_preferences() (writes Language::code() → "language=…")
            → install_menus(self.language, cx)  // retranslate native menus
            → status = StatusLanguageSet
            → close any open dropdown
```

The cached-per-version Markdown pipeline (`model::DocumentState`, preview blocks, outline, stats) is **not** on this path and is untouched by the change. `builtin_theme_definitions()` is a static table consulted at render time when building the theme swatch grid; it does not participate in per-keystroke recomputation.

## Migration Plan

- **Persistence:** additive only. A new `language=…` line is written. Older builds ignore it; this build reads its absence as English default. No migration of existing files is required.
- **Rollout:** single commit boundary — i18n module + model/storage changes + main.rs wiring + View → Language submenu, all in one change. No partial state that needs guarding.
- **Rollback:** revert the change; existing preferences files remain valid (the `language=` line is tolerated/ignored by older code paths). Theme names are unchanged, so saved theme selections are preserved across rollback.

## Open Questions

- Should the exhaustiveness test be replaced by a declarative macro that yields both the `Msg` list and the per-language tables from one source of truth? (Out of scope for this change; tracked as a follow-up.)
- Is a third language needed before v1? Current scope is English + Simplified Chinese; the enum is extensible.
