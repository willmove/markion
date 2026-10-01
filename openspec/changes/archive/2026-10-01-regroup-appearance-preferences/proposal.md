## Why

The Preferences panel already isolates theme selection on its own tab, but the label still says **Theme** while fonts, sizes, and paragraph spacing stay on **General** next to language and auto-save. Users looking for how the document looks have to hunt across tabs, and “Theme” under-describes a surface that should hold every appearance control.

## What Changes

- Rename the Preferences **Theme** tab to **Appearance** in every supported UI language (English “Appearance”, Simplified Chinese “外观”, and the matching compact labels in the other five locales).
- Move the existing typography section onto that tab: Source font size, Reading font size, Paragraph spacing, and the three font-family slots (source, rendered, code).
- Keep language, display/workspace toggles (focus, typewriter, line numbers, Preview adaptive width, heading-menu depth, Sync scroll, hidden files, open-in-current-tab, sidebar), and Auto-save on **General**. Theme swatch apply/persist is unchanged.
- Tab order stays General, Appearance, Shortcuts, Export. File → Preferences still lands on General.
- Point user docs that currently say **Preferences → Theme** at **Preferences → Appearance**.

**Non-goals:** no new appearance controls, no `config.toml` schema change, no theme-catalog or custom-theme authoring work, no moving language/display/auto-save/shortcut/export onto Appearance, and no change to Cycle Theme or the Help preferences summary.

This change does not touch per-document-version Markdown caches, syntax-highlight memoization, or the cached text handle.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `theme-preferences`: The Theme tab becomes an Appearance tab that hosts both the swatch grid and document typography (sizes, spacing, font families); General no longer hosts those appearance controls.
- `chrome-platform`: The Appearance tab body is the named Preferences scroll region that previously belonged to Theme.
- `ui-i18n`: The Appearance tab label (and any remaining Appearance-tab chrome) is routed through the i18n layer in every supported language.
- `project-documentation`: FAQ (and any README navigation claims) describe theme and typography as **Preferences → Appearance**.

## Impact

- Affected code: `src/app/mod.rs` (`PreferencesTab`, scroll target), `src/app/application.rs`, `src/app/root_view.rs` (tab strip, General body, Appearance body), `src/app/appearance.rs` (sync-scroll no-op for the renamed target), `src/app/tests.rs`, `src/i18n.rs`.
- Docs: `docs/faq.md` Themes section currently points at **Preferences → Theme**; bilingual READMEs if they name the Theme tab.
- Persistence, theme catalog, font-resolution order, Markdown derived-state caches, syntax highlighting, and the cached text handle are untouched.
- Builds on the already-implemented Theme tab from `extract-theme-preferences-tab` (sibling tab, 640px shell, dedicated scroll handle). Four tab labels must still fit that shell.
