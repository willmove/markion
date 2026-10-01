## Context

The Preferences panel (`preferences_panel_view` in `src/app/root_view.rs`) already uses a sibling tab strip: `PreferencesTab::{General, Shortcuts, Export}`. Theme selection is not a tab; it is a swatch-grid section rendered inside the General body, between Language and Typography. General has grown (language, theme, typography, display, auto-save), so theme — a catalog with 14+ built-ins plus custom files — competes with unrelated controls on the default tab.

Theme apply/persist (`apply_theme_by_name`, `config.toml` `theme=`), the built-in catalog, custom `.toml` loading, and sample-theme install on Preferences open stay as they are. This change is panel navigation only.

## Goals / Non-Goals

**Goals:**

- Promote theme selection to a sibling Preferences tab, same level as General, Shortcuts, and Export.
- Move the existing swatch grid onto that tab without changing apply, persist, or catalog behavior.
- Keep General as the default landing tab, with language / typography / display / auto-save still there.
- Give the Theme tab the same draggable scrollbar contract as the other Preferences bodies.
- Localize the new tab label in all seven UI languages.

**Non-Goals:**

- Moving language, font-family slots, typography sizes, or display toggles onto the Theme tab.
- A theme editor, new catalog entries, or any `config.toml` format change.
- Changing Cycle Theme, the Help preferences summary, or F1 → Shortcuts.
- Widening the panel for Theme (Shortcuts stays the only 720px tab).

This change does not touch Markdown derived-state caches, syntax-highlight memoization, or the per-version text handle.

## Decisions

### 1. Tab order: General, Theme, Shortcuts, Export

Theme sits immediately after General because it is extracted from that tab; Shortcuts and Export keep their relative order (F1 still opens Shortcuts). Alternatives considered: Theme last (hides a frequent appearance choice) and Theme before General (breaks the existing default-first convention).

### 2. Extract `preferences_theme_body`, mirror Export

General currently inlines the swatch grid. Export already lives in `preferences_export_body` with its own scroll handle and `PaneScrollTarget`. Theme follows that pattern:

- `PreferencesTab::Theme`
- `preferences_theme_body` holding the current swatch-grid markup (including `PrefPanelThemeSection`)
- `preferences_theme_scroll` + `PaneScrollTarget::PreferencesTheme` (sync-scroll no-op, like the other Preferences targets)
- 640px shell, same as General/Export

Keeping the inner section heading avoids a one-off layout and reuses the already-translated `PrefPanelThemeSection` string.

### 3. Sample theme install stays on Preferences open

`show_preferences` already calls `ensure_sample_custom_theme()` before showing the panel and resets `preferences_tab` to General. Leave that on panel open, not Theme-tab select, so first-use `typewriter.toml` install does not depend on visiting Theme.

### 4. New `Msg::PrefPanelTabTheme`; reuse existing Theme wording

Add a tab-label message and translate it in every language block to the same word already used for `PrefPanelThemeSection` (en Theme, zh-Hans 主题, ja テーマ, fr Thème, de Design, es Tema, zh-Hant 佈景主題). Do not hard-code the tab label.

### 5. Docs point at Preferences → Theme

`docs/faq.md` currently says to pick a theme in the Preferences panel. Point that line at the Theme tab so it matches Shortcuts (`Preferences → Shortcuts`) and Auto-save (`Preferences → General → Auto-save`).

## Risks / Trade-offs

- [Four tab labels overflow in long languages] → Reuse the existing compact `preferences_tab_button` styling; German “Allgemein / Design / Tastenkürzel / Exportieren” is the tightest case and should still fit 640px. If it wraps, shrink padding rather than dropping a tab.
- [Source-string tests that key off `// Theme grid.`] → The language-picker test splits General on that comment. After the move, retarget the seam at Typography (or the extracted `preferences_theme_body`) so the test still pins wrap/nowrap behavior.
- [Users look for theme on General] → Tab strip order puts Theme next to General; Cycle Theme and persistence are unchanged.

## Migration Plan

UI-only. No preferences-file migration. Rollback is reverting the panel split; stored `theme=` values remain valid.

## Open Questions

None. Scope is a navigation split of an existing control.
