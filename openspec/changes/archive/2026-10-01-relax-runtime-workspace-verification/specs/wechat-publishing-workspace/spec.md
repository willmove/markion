## ADDED Requirements

### Requirement: Publishing launches after a minimal runtime gate

Launching the local WeChat publishing workspace SHALL require only a minimal runtime gate: the
bundle manifest is readable and parses, its provenance metadata is valid, and the entry shell
`index.html` exists and matches its manifest-recorded LF-normalized SHA-256 digest. The launch
path SHALL NOT require whole-tree bundle verification and SHALL NOT fail because files exist on
disk that the manifest does not list, because manifest-listed files other than the entry shell
fail their digest check, or because release-only scans (remote runtime dependencies, prohibited
export artifacts) would reject the directory. Full-bundle verification SHALL remain available and
SHALL stay exhaustive for release construction, pre-publication checks, and maintainer tooling.

#### Scenario: Upgrade leftovers do not block launching

- **WHEN** the installed workspace directory still contains files that a newer package no longer
  ships (for example, files removed from the bundle between two released versions, left behind by
  an in-place package upgrade), and the user opens the WeChat publishing workspace
- **THEN** the workspace session is created and the default browser opens to it
- **AND** the launch does not attempt to delete or modify the leftover files

#### Scenario: Missing manifest or invalid provenance blocks launching

- **WHEN** the workspace directory's manifest is missing, unparseable, or fails provenance
  validation
- **THEN** workspace setup fails with a setup error status and no browser session is created

#### Scenario: Tampered or missing entry shell blocks launching

- **WHEN** `index.html` is absent from disk, absent from the manifest, or its bytes do not match
  the manifest digest after LF normalization
- **THEN** workspace setup fails with a setup error status naming the file and no browser session
  is created

#### Scenario: Release verification stays exhaustive

- **WHEN** the release pipeline or the `verify-bundle` maintainer CLI verifies a workspace
  directory that contains an unlisted file, a missing file, or a digest mismatch on any listed
  file
- **THEN** full-bundle verification still fails before publication
- **AND** the strictness of release-time checks is not reduced by the relaxed runtime gate
