# branding-assets Specification

## Purpose
Govern the project's reusable branding assets — the canonical SVG logo and the platform icon derivatives (.ico/.icns/.png) consumed by packaging, the Windows executable resource, and documentation — so every consumer draws from a single source rather than ad-hoc copies.
## Requirements
### Requirement: Reusable SVG logo asset
Markion SHALL provide a self-contained SVG logo asset suitable for documentation, packaging, and future application branding.

#### Scenario: Logo is available as vector art
- **WHEN** project branding needs a scalable icon
- **THEN** `assets/markion-logo.svg` provides the logo as valid SVG with an explicit `viewBox`

#### Scenario: Simplified logo variant is available
- **WHEN** project branding needs a simpler modern icon
- **THEN** `assets/markion-logo-simple.svg` provides a valid SVG variant with an explicit `viewBox`

### Requirement: Platform icon exports
Markion SHALL provide rendered icon assets for README display and desktop platform packaging.

#### Scenario: PNG icon is available
- **WHEN** documentation or packaging needs a raster preview icon
- **THEN** `assets/markion.png` provides a PNG export of the simplified logo

#### Scenario: Windows icon is embedded by the build
- **WHEN** Markion is built on Windows
- **THEN** `assets/markion.ico` is embedded as the executable/window icon resource

#### Scenario: macOS icon asset is available
- **WHEN** a macOS app bundle needs a Dock icon
- **THEN** `assets/markion.icns` provides an ICNS export of the simplified logo
