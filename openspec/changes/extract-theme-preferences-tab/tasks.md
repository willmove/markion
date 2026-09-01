## 1. Tab plumbing and localization

- [x] 1.1 Add `PreferencesTab::Theme` in `src/app/mod.rs`, plus `PaneScrollTarget::PreferencesTheme` and a `preferences_theme_scroll` handle initialized in `src/app/application.rs`
- [x] 1.2 Treat `PreferencesTheme` as a sync-scroll no-op in `src/app/appearance.rs` and as a named scrollbar id in `pane_scrollbar_view`
- [x] 1.3 Add `Msg::PrefPanelTabTheme` with translations in all seven language blocks (same wording as `PrefPanelThemeSection`) and include it in the exhaustive i18n message list

## 2. Preferences panel Theme tab

- [x] 2.1 Extract the General-tab swatch grid into `preferences_theme_body` (keep `PrefPanelThemeSection` and the existing card markup/apply path) with the same scrollable wrapper as Export (`#preferences-theme-body`, `preferences_theme_scroll`, `PaneScrollTarget::PreferencesTheme`)
- [x] 2.2 Wire the tab strip as General, Theme, Shortcuts, Export; render `preferences_theme_body` when `PreferencesTab::Theme` is active; leave File → Preferences landing on General
- [x] 2.3 Confirm General still starts with Language then Typography / Other / Auto-save and no longer contains the swatch grid

## 3. Tests and docs

- [x] 3.1 Add a source-string test that the tab strip and panel body wire `PreferencesTab::Theme` / `Msg::PrefPanelTabTheme` / `preferences_theme_body`, and that General no longer inlines the swatch grid
- [x] 3.2 Retarget `preferences_language_picker_contains_variable_width_labels` so it no longer splits on `// Theme grid.`; extend scrollbar-handle tests to cover `PreferencesTheme`
- [x] 3.3 Update `docs/faq.md` Themes guidance to **Preferences → Theme**
- [x] 3.4 Run `cargo test --workspace` and `openspec validate extract-theme-preferences-tab`
