## Why

Markion currently hard-codes document text sizes and rendered paragraph gaps, so users cannot adapt the workspace for different displays, reading distances, or accessibility needs. Exposing these values as persisted appearance preferences makes every view mode more comfortable without changing Markdown content.

## What Changes

- Add independently configurable source-editor and rendered-document font sizes that apply immediately across Edit, Visual Edit, Split Preview, and Read mode while preserving the current appearance as the defaults.
- Add a rendered paragraph-spacing preference that applies immediately to Visual Edit and rendered preview/read content; the source editor continues to represent paragraph separation from literal Markdown lines.
- Add localized numeric controls to the Preferences panel and persist validated values in `config.toml`, including defaulting, reset, and legacy/missing-field compatibility.
- Route typography values through existing GPUI layout and paint paths so text reflows, selection/caret geometry, scrolling, inline math, and virtualized block measurement remain consistent without invalidating or recomputing derived Markdown state.
- Non-goals: font-family selection, per-document or per-tab overrides, changing authored Markdown whitespace, and applying app typography preferences to exported HTML/PDF/DOCX/images.

## Capabilities

### New Capabilities

- `document-typography`: Define how configurable source/rendered font sizes and paragraph spacing affect the source, visual-edit, preview, and read surfaces while preserving document and cache invariants.

### Modified Capabilities

- `theme-preferences`: Add localized numeric typography controls and their persisted/reset preference contract.
- `chrome-platform`: Extend the supported preference set to include document font size and rendered paragraph spacing instead of treating font size as non-configurable.

## Impact

- Preferences model and TOML storage in `src/model.rs` and `src/storage/preferences.rs`, plus app initialization/reset/current-preference plumbing.
- Preferences-panel controls and translations in `src/app/root_view.rs`, `src/app/appearance.rs`, and `src/i18n.rs`.
- Source editor, Visual Edit, preview/read, inline-math sizing, list measurement, scroll, caret, and selection layout under `src/app/`.
- Focused persistence/layout tests and the existing workspace test suite; no new dependency or file-format migration is required because all new TOML fields are optional and defaulted.
