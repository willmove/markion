## Context

The Preferences panel already uses a sibling tab strip: `PreferencesTab::{General, Theme, Shortcuts, Export}` (the Theme tab landed in `extract-theme-preferences-tab`). Theme selection lives in `preferences_theme_body`; document typography (source/reading sizes, paragraph spacing, three font-family slots) is still a section of the General body, between Language and Other.

Apply/persist paths are unchanged: `apply_theme_by_name` + `theme=` for the swatch grid; `set_editor_font_size` / `set_rendered_font_size` / `set_paragraph_spacing` and the font-slot helpers for typography. Font resolution (explicit preference → theme `[fonts]` → built-in default) and presentation-only reflow (no Markdown version bump, no derived-cache rebuild) stay as they are. This change is panel navigation and labeling only.

## Goals / Non-Goals

**Goals:**

- Rename the Theme tab to Appearance in every supported UI language.
- Move the existing typography section onto that tab so theme + fonts/sizes/spacing share one appearance surface.
- Leave language, display/workspace toggles, and Auto-save on General.
- Keep File → Preferences landing on General; keep tab order General, Appearance, Shortcuts, Export.
- Give the Appearance body the same draggable scrollbar contract the Theme body already has.
- Point FAQ / README navigation claims at **Preferences → Appearance**.

**Non-Goals:**

- New appearance controls, a theme editor, catalog changes, or any `config.toml` format change.
- Moving language, focus/typewriter, line numbers, Preview adaptive width, heading-menu depth, Sync scroll, hidden files, open-in-current-tab, sidebar, Auto-save, Shortcuts, or Export onto Appearance.
- Changing Cycle Theme, the Help preferences summary, or F1 → Shortcuts.
- Widening the panel (Shortcuts stays the only 720px tab).

This change does not touch Markdown derived-state caches, syntax-highlight memoization, or the per-version text handle. Typography apply still refreshes presentation layout only.

## Decisions

### 1. Rename the tab identity, not just the label

`PreferencesTab::Theme` becomes `PreferencesTab::Appearance`, with matching names for the body helper, scroll handle, and `PaneScrollTarget`. Keeping `Theme` as the identifier while hosting fonts would leave tests and comments describing the wrong surface.

Alternatives considered: change only `Msg::PrefPanelTabTheme` copy (less churn, but the enum would still say Theme); introduce a fifth tab (rejected — Appearance is the Theme tab expanded).

### 2. Appearance body: theme swatches, then typography

`preferences_theme_body` is renamed to `preferences_appearance_body` and gains the typography block currently inlined in General (numeric size/spacing rows plus `preference_font_row` for the three slots). Inner headings stay `PrefPanelThemeSection` and `PrefPanelTypographySection`. Theme stays first because it is the visual identity and the original tab content; typography follows as the rest of appearance.

The font picker is already inline in `preference_font_row` (`app.font_picker`); it does not depend on General being active. `show_preferences` continues to close the picker and land on General.

### 3. General keeps Language / Other / Auto-save

Display and workspace toggles (focus, typewriter, line numbers, adaptive width, heading-menu depth, Sync scroll, hidden files, open-in-current-tab, sidebar) are editor behavior, not document appearance. Adaptive width and line numbers are the closest calls; they stay on General so Appearance stays theme + type.

### 4. New `Msg::PrefPanelTabAppearance`; retire the Theme tab label

Replace `PrefPanelTabTheme` with `PrefPanelTabAppearance` in every language block and the exhaustive i18n list. Compact labels:

| Locale | Tab label |
| --- | --- |
| en | Appearance |
| zh-Hans | 外观 |
| zh-Hant | 外觀 |
| ja | 外観 |
| fr | Apparence |
| de | Darstellung |
| es | Apariencia |

German uses “Darstellung” (same word VS Code uses for Appearance) rather than “Erscheinungsbild”, so four tabs still fit the 640px shell. Do not hard-code the tab label. Keep `PrefPanelThemeSection` as the inner Theme heading.

### 5. Scroll target rename

`preferences_theme_scroll` / `PaneScrollTarget::PreferencesTheme` become `preferences_appearance_scroll` / `PreferencesAppearance`. Sync-scroll remains a no-op for that target, same as the other Preferences regions. The 640px shell and `#preferences-theme-body` id become an appearance-named id so scrollbar tests stay aligned.

### 6. Docs point at Preferences → Appearance

`docs/faq.md` currently says **Preferences → Theme**. Point theme picking (and any typography navigation) at **Preferences → Appearance**. Bilingual READMEs that list theme and per-plane fonts as undifferentiated panel items should describe them as Appearance preferences.

## Risks / Trade-offs

- [Four tab labels overflow in long languages] → Keep the existing compact `preferences_tab_button` styling. German “Allgemein / Darstellung / Tastenkürzel / Exportieren” is the tightest case; if it wraps, shrink padding rather than dropping a tab.
- [Users still look for typography on General] → Tab order puts Appearance next to General; inner Typography heading is unchanged.
- [Overlap with unarchived `extract-theme-preferences-tab`] → That change forbade typography on the Theme tab. This change supersedes that grouping. Implement against the current code (Theme tab already exists); archive extract first if both land.
- [Source-string tests keyed on `PreferencesTab::Theme` / `preferences_theme_body`] → Retarget those asserts to Appearance names and confirm General no longer inlines typography.

## Migration Plan

UI-only. No preferences-file migration. Rollback is reverting the panel regroup; stored `theme=` and typography keys remain valid.

## Open Questions

None. Display toggles stay on General unless a follow-up asks to move Preview adaptive width or code line numbers.
