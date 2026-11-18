## MODIFIED Requirements

### Requirement: Reusable SVG logo asset
Markion SHALL provide `assets/markion-logo.svg` as a self-contained, valid SVG with an explicit `viewBox`. It SHALL be the canonical source for documentation, packaging, and generated desktop icon assets. The artwork SHALL display the Page Fold M, a light capital M with inward-curving page edges and a clipped upper-right corner on a deep teal rounded square. The area outside the square SHALL be transparent.

#### Scenario: Logo is available as vector art
- **WHEN** project branding needs a scalable icon
- **THEN** `assets/markion-logo.svg` provides the Page Fold M as valid SVG vector artwork with an explicit `viewBox`

#### Scenario: Canonical artwork is used for generated assets
- **WHEN** the project icon-generation script runs
- **THEN** it rasterizes `assets/markion-logo.svg` rather than an alternate or obsolete logo file
