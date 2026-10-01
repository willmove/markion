## 1. Tab identity and localization

- [x] 1.1 Rename `PreferencesTab::Theme` to `Appearance` in `src/app/mod.rs`, plus `PaneScrollTarget::PreferencesTheme` → `PreferencesAppearance` and `preferences_theme_scroll` → `preferences_appearance_scroll` (init in `src/app/application.rs`)
- [x] 1.2 Treat `PreferencesAppearance` as a sync-scroll no-op in `src/app/appearance.rs` and as a named scrollbar id in `pane_scrollbar_view`; update any remaining `PreferencesTheme` / `preferences_theme_scroll` references
- [x] 1.3 Replace `Msg::PrefPanelTabTheme` with `PrefPanelTabAppearance` and translate in all seven language blocks (en Appearance, zh-Hans 外观, zh-Hant 外觀, ja 外観, fr Apparence, de Darstellung, es Apariencia); include it in the exhaustive i18n message list. Keep `PrefPanelThemeSection` as the inner Theme heading

## 2. Preferences panel Appearance body

- [x] 2.1 Rename `preferences_theme_body` to `preferences_appearance_body`, keep the swatch grid first, then move the General-tab typography block (size/spacing numeric rows and the three `preference_font_row` slots) into that body with the existing Appearance scroll wrapper
- [x] 2.2 Wire the tab strip as General, Appearance, Shortcuts, Export using `Msg::PrefPanelTabAppearance`; render `preferences_appearance_body` when `PreferencesTab::Appearance` is active; leave File → Preferences landing on General
- [x] 2.3 Confirm General still starts with Language then Other / Auto-save and no longer contains the swatch grid or typography controls; font picker continues to work from the Appearance tab

## 3. Tests and docs

- [x] 3.1 Retarget source-string tests from `PreferencesTab::Theme` / `Msg::PrefPanelTabTheme` / `preferences_theme_body` to the Appearance names, and assert General no longer inlines the swatch grid or typography section
- [x] 3.2 Extend scrollbar-handle tests to cover `PreferencesAppearance`; keep language-picker wrap tests on the General body
- [x] 3.3 Update `docs/faq.md` to **Preferences → Appearance**; update `README.md` and `README.zh-CN.md` so theme and document typography are described as Appearance preferences
- [x] 3.4 Run `cargo test --workspace` and `openspec validate regroup-appearance-preferences`
