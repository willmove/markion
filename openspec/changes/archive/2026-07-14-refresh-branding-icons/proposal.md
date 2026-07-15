## Why

The canonical Markion logo has moved to `assets/markion-logo.svg`, while the generated desktop icon files and documentation still reference obsolete asset names. Regenerating and wiring the platform derivatives keeps the Windows executable, macOS Dock, Linux packaging, and README visually consistent.

## What Changes

- Make `assets/markion-logo.svg` the sole source artwork for generated branding exports.
- Update the icon-generation script to create `markion.png`, `markion.ico`, and `markion.icns` from that SVG.
- Embed the ICO in Windows builds and configure the README and packaging metadata to consume the canonical exports.

Non-goals: redesigning the logo, changing the product identity, or adding code signing.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `branding-assets`: define the canonical SVG source and its generated platform derivatives.
- `release-packaging`: ensure packaged platform artifacts consume the generated icons.

## Impact

Affected files include `openspec/scripts/generate-icons.mjs`, `assets/`, `build.rs`, `Cargo.toml`, package metadata, and `README.md`. The change preserves editor runtime and derived-state caching invariants.
