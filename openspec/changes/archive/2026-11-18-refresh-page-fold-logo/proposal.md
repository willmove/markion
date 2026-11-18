## Why

The approved Page Fold M gives Markion a more distinctive document-editor identity while retaining the initial from the existing logo. Documentation and desktop entry points should show the same mark.

## What Changes

- Rebuild the approved Page Fold M as the canonical `assets/markion-logo.svg` artwork.
- Regenerate `assets/markion.png`, `assets/markion.ico`, and `assets/markion.icns` from that SVG.
- Verify the mark at desktop icon sizes and confirm that the existing asset consumers resolve the refreshed files.

Non-goals include changing the product name, packaging settings, or publishing a release.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `branding-assets` specifies the approved artwork used by the canonical logo and its exports.

## Impact

The four branding assets change in place. Documentation, packaging, and the Windows resource build continue to use the canonical paths. The change does not touch editor runtime or derived state.
