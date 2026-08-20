## ADDED Requirements

### Requirement: Packaged local WeChat publishing workspace
Every supported Markion native package SHALL include the complete pinned MarkNice publishing workspace, its locally hosted third-party runtime assets and fonts, a machine-readable provenance manifest, and applicable license notices. Release construction and verification SHALL require neither a sibling MarkNice checkout nor Node.js, SHALL verify that the bundled application shell has no remote runtime dependency, and SHALL fail before publication when required workspace assets are absent, unlisted, or inconsistent with their manifest.

#### Scenario: Native packages contain the workspace
- **WHEN** the Windows NSIS, macOS Apple Silicon application/DMG, Linux amd64 DEB, or Linux x86_64 AppImage is assembled
- **THEN** the package contains the same required publishing workspace files at the runtime resource location expected by Markion

#### Scenario: Package build is self-contained
- **WHEN** a release package is built from a clean Markion checkout without the sibling MarkNice repository and without Node.js
- **THEN** the checked-in pinned workspace is packaged successfully
- **AND** no build step downloads a MarkNice or CDN runtime asset

#### Scenario: Offline bundle verification succeeds
- **WHEN** the release verification inspects the packaged publishing shell and manifest
- **THEN** every required file is present and matches its recorded identity
- **AND** scripts, styles, fonts, and renderer dependencies resolve to packaged local assets rather than remote URLs

#### Scenario: Incomplete bundle blocks publication
- **WHEN** a required workspace file, provenance entry, or applicable third-party notice is missing or inconsistent
- **THEN** packaging or release verification fails before the release is published

