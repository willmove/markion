## ADDED Requirements

### Requirement: Release publication SHALL follow a repeatable verified procedure
The project SHALL document and follow a canonical GitHub release procedure that selects a non-conflicting semantic version, synchronizes every repository-controlled version field, validates the workspace, creates a dedicated release commit and annotated version tag, pushes the default branch and tag, monitors the tag-triggered workflow through completion, and verifies the final GitHub Release before reporting success. When the requester does not specify a version, the procedure SHALL default to the next patch version after the highest stable version tag. A public tag SHALL NOT be deleted, force-moved, or recreated without explicit authorization.

#### Scenario: Routine release has no requested version
- **WHEN** a maintainer requests a new release without naming a version
- **THEN** the operator selects the next patch version after the highest stable `vMAJOR.MINOR.PATCH` tag
- **AND** verifies that neither that tag nor its GitHub Release already exists

#### Scenario: Version metadata is synchronized before tagging
- **WHEN** a release version is prepared
- **THEN** the workspace and root package versions in `Cargo.toml`, the packaging version in `packager.toml`, and the affected workspace entries in `Cargo.lock` resolve to the same version
- **AND** `cargo metadata --no-deps` confirms that every Markion workspace package uses that version

#### Scenario: Validation fails before publication
- **WHEN** `cargo test --workspace`, version validation, or release-diff validation fails
- **THEN** no release tag is pushed and the release is not reported as published

#### Scenario: Tag workflow is the publication gate
- **WHEN** the release commit and annotated version tag are pushed
- **THEN** the operator monitors the tag-triggered GitHub Actions run until every required native build, package upload, and release job succeeds
- **AND** verifies the final Release and required assets before reporting completion

#### Scenario: A public tagged release encounters a failure
- **WHEN** publication fails after the version tag is visible on GitHub
- **THEN** the operator reports the failed stage and preserves the public tag unless explicit authorization is given for a destructive correction

### Requirement: Published releases SHALL contain curated release information
The final GitHub Release description SHALL expand or replace auto-generated notes with a structured summary derived from the commits, diff, and completed OpenSpec changes since the previous tag. Unless the requester specifies another language, the summary SHALL be written in Simplified Chinese and SHALL cover user-visible highlights and fixes, compatibility or migration information, available platform downloads, verification results, and a full comparison link. The final Release SHALL be a non-draft stable release unless a prerelease was explicitly requested.

#### Scenario: Generated notes omit direct commits
- **WHEN** GitHub's generated notes mention only merged pull requests or otherwise omit user-visible work
- **THEN** the operator supplements or replaces them with the complete curated summary before reporting the release complete

#### Scenario: Release has no migrations
- **WHEN** a version changes no persisted Markdown, preferences, or workspace data formats
- **THEN** the compatibility section explicitly states that no migration is required
- **AND** retains the documented unsigned-installer warning when applicable

#### Scenario: Final release information is verified
- **WHEN** the tag workflow succeeds
- **THEN** the operator confirms that the Release is neither a draft nor an unintended prerelease
- **AND** confirms that the Windows NSIS installer, macOS Apple Silicon DMG, Linux amd64 DEB, and Linux x86_64 AppImage are attached
- **AND** confirms that the curated notes and comparison link are present
