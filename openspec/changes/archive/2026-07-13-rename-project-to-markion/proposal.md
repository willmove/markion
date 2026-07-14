## Why

The project is adopting the Markion name. Its prior branding is embedded in the crate, application metadata, identifiers, assets, documentation, and user-facing UI, which would otherwise leave the delivered product inconsistent.

## What Changes

- **BREAKING** Rename the root Cargo package and binary to `markion`.
- **BREAKING** Replace every prior product-facing name and case-sensitive machine identifier with its Markion/markion equivalent, including packaging metadata, bundle identifiers, executable resources, assets, configuration and storage locations, and documentation.
- Rename canonical logo and platform-icon asset filenames to `markion-*` and update every consumer.
- Preserve the editor's behavior and its cached derived-Markdown-state invariants; this change changes branding and identifiers only.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `branding-assets`: Canonical logo and icon asset names and the branded application name change to Markion.
- `chrome-platform`: Preferences and diagnostic log directory names change to Markion.
- `code-and-math`: The syntax-highlighting theme-class reference changes to Markion.
- `release-packaging`: Release product name, bundle identifier, executable, and icon references change to Markion.
- `crate-architecture`: The root application package and default Cargo command target become `markion`.

## Impact

Affected areas include the root Cargo manifest and lockfile, build and packager metadata, Rust source constants and UI text, asset filenames, documentation, examples, scripts, and OpenSpec records. No runtime dependencies or editor behavior change.
