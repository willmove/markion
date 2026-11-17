## Why

The 32px paragraph-spacing ceiling limits readers who prefer a more open layout, and document line spacing is currently fixed. Users need both controls in Appearance preferences without changing their Markdown or losing their reading position.

## What Changes

- Expand rendered paragraph spacing from 0–32px to 0–128px, keeping the 12px default and 1px controls.
- Add a global line-spacing preference of 75–300% relative to the current layout, default 100%, with localized 5-point controls and saved configuration.
- Apply line-spacing changes consistently to source, Visual Edit, Split Preview and Read document text, including code and table geometry; keep application chrome and exports unchanged.
- Preserve typography-only behavior: document versions, dirty state, undo/redo, selections, Markdown-derived caches, highlighting and cached text handles remain intact, and affected layout is remeasured around its scroll anchor.

Non-goals: changing document/export content, redesigning the preferences input widgets, per-document overrides, or altering font defaults. No new dependencies.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `document-typography`: wider paragraph gaps and adjustable document line-height scaling with coherent editing geometry.
- `theme-preferences`: localized line-spacing controls, persisted percentage, backward-compatible defaults, bounds and reset.

## Impact

Preference model/serialization, typography metric resolution, source/rendered layout measurements, Appearance controls, all eight translation catalogs, and focused typography/persistence/layout tests. Existing per-version derived Markdown and text-handle caches must not be invalidated by these presentation-only settings.
