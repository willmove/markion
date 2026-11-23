## Why

Markion v0.6.2 fixed the table header, body, and border colors from #27. The drop-target fill, faint cell selections, destructive menu actions, and collapsible source editor chrome still need to follow the active theme.

## What Changes

- Cache the resolved palette when the app loads or applies a theme, and refresh it when the active custom theme's colors change.
- Preserve theme colors that pass the contrast and visibility checks. Use fixed, polarity-aware fallback mixes for table selection and drag targets, and shared colors for controls and destructive actions.
- Apply shared source editor styling to the collapsible HTML, math, YAML front matter, and diagram editors in Visual Edit.
- Derive the colors for all 14 built-in themes and for existing full and sparse custom theme definitions.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tables-outline` covers table surface and interaction colors in Visual Edit, Read, and Split Preview.
- `markdown-editing` covers the shared theme colors used by collapsible source editors.

## Impact

- `src/app/mod.rs` resolves table and control colors. `src/app/application.rs` and `src/app/appearance.rs` populate and refresh the palette cache.
- `src/app/preview.rs` applies the cached palette to tables, controls, menus, and collapsible source editors.
- `src/app/tests.rs`: unit and GPUI tests across all 14 built-in themes, custom full/sparse `.toml` themes, live theme switching, and all three rendered modes (`VisualEdit`, `Read`, `Split`).
- Preserves all document-version, `Arc`-shared derived Markdown caches, memoized highlighting, text-handle caching, and undo invariants; theme switching affects presentation only.
