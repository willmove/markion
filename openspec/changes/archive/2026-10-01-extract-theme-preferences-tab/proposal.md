## Why

Theme selection currently lives as a section inside the Preferences **General** tab, mixed with language, typography, display, and auto-save controls. Theme is a first-class appearance choice with its own swatch catalog, and it should sit at the same navigation level as General, Shortcuts, and Export instead of being buried in General.

## What Changes

- Add a dedicated **Theme** tab to the Preferences panel, at the same tab-strip level as General, Shortcuts, and Export.
- Move the theme swatch grid (built-in plus custom themes, live apply, persistence) out of General and onto that Theme tab. Selecting a card still applies immediately and writes the existing `theme=` preference.
- Leave language, typography (including font-family slots), display toggles, and auto-save on General. Opening Preferences still lands on General by default.
- Localize the new tab label in every supported UI language. Give the Theme tab body the same draggable right-side scrollbar as the other Preferences scroll regions.
- Update user docs that currently describe picking a theme from an undifferentiated Preferences panel.

Non-goals: no new theme catalog, custom-theme authoring UI, persistence-format change, or moving language/typography/display settings onto the Theme tab. Cycle Theme and the Help preferences summary stay as they are.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `theme-preferences`: Theme swatch selection becomes a sibling Preferences tab rather than a General-tab section; the Language-before-Theme ordering requirement is replaced by tab-level navigation.
- `chrome-platform`: The Theme tab body is a scrollable Preferences region with the same draggable scrollbar contract as General, Shortcuts, and Export.
- `ui-i18n`: The Theme tab label (and any remaining Theme-tab chrome) is routed through the i18n layer in every supported language.

## Impact

- Affected code: `src/app/mod.rs` (`PreferencesTab`, scroll target/handle), `src/app/application.rs`, `src/app/root_view.rs` (tab strip, General body, new Theme body), `src/app/appearance.rs` (sync-scroll no-op for the new target), `src/app/tests.rs`, `src/i18n.rs`.
- Docs: `docs/faq.md` (Themes section currently points at an undifferentiated Preferences panel).
- Persistence, theme catalog, Markdown derived-state caches, syntax highlighting, and the cached text handle are untouched.
- No new dependencies. Four tab labels must still fit the existing 640px Preferences shell.
