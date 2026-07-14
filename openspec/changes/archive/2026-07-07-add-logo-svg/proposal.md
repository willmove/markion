## Why

Markion does not yet have a reusable project logo asset for documentation, packaging, or future application branding. Adding a standalone SVG gives the project a crisp, scalable visual identity without changing runtime behavior.

## What Changes

- Add a vector logo icon that combines a Markdown document shape, an `M` mark, and an editing cursor accent.
- Add a simplified modern SVG logo variant and export platform-ready PNG, ICO, and ICNS icon assets.
- Wire the Windows icon into the build resource pipeline and set Markion window metadata during GPUI window creation.
- Add the simplified logo to the top of the README.
- Keep the asset self-contained as plain SVG so it can be reused in docs, installers, or future app icon generation.
- Non-goals: this change does not add a full installer/package bundle or change editor runtime behavior beyond window metadata.

## Capabilities

### New Capabilities
- `branding-assets`: Reusable project identity assets for documentation, packaging, and future app branding.

### Modified Capabilities
- None.

## Impact

- Adds `assets/markion-logo.svg`, `assets/markion-logo-simple.svg`, `assets/markion.png`, `assets/markion.ico`, and `assets/markion.icns`.
- Adds a Windows-only build dependency for embedding `assets/markion.ico` as the executable/window icon.
- Updates README branding and GPUI window initialization metadata.
- No APIs, user-visible strings, or cached Markdown-state invariants are touched.
