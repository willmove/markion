# branding-assets Specification

## Purpose
Govern the project's reusable branding assets — the canonical SVG logo and the platform icon derivatives (.ico/.icns/.png) consumed by packaging, the Windows executable resource, and documentation — so every consumer draws from a single source rather than ad-hoc copies.
## Requirements
### Requirement: Reusable SVG logo asset
Markion SHALL provide `assets/markion-logo.svg` as a self-contained, valid SVG with an explicit `viewBox`; it SHALL be the canonical source for documentation, packaging, and generated desktop icon assets.

#### Scenario: Logo is available as vector art
- **WHEN** project branding needs a scalable icon
- **THEN** `assets/markion-logo.svg` provides the logo as valid SVG with an explicit `viewBox`

#### Scenario: Canonical artwork is used for generated assets
- **WHEN** the project icon-generation script runs
- **THEN** it rasterizes `assets/markion-logo.svg` rather than an alternate or obsolete logo file

### Requirement: Platform icon exports
Markion SHALL provide rendered icon assets generated from `assets/markion-logo.svg` for README display and desktop platform packaging.

#### Scenario: PNG icon is available
- **WHEN** documentation or Linux packaging needs a raster preview icon
- **THEN** `assets/markion.png` provides a 512×512 PNG export of the canonical logo

#### Scenario: Windows icon is embedded by the build
- **WHEN** Markion is built on Windows
- **THEN** `assets/markion.ico` is embedded as the executable/window icon resource

#### Scenario: macOS icon asset is available
- **WHEN** a macOS app bundle needs a Dock icon
- **THEN** `assets/markion.icns` provides an ICNS export of the canonical logo

