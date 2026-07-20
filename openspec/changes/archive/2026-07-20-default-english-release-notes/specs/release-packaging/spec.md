## MODIFIED Requirements

### Requirement: Published releases SHALL contain curated release information
The final GitHub Release description SHALL expand or replace auto-generated notes with a structured summary derived from the commits, diff, and completed OpenSpec changes since the previous tag. Unless the requester specifies another language, the summary SHALL be written in English and SHALL cover user-visible highlights and fixes, compatibility or migration information, available platform downloads, verification results, and a full comparison link. The final Release SHALL be a non-draft stable release unless a prerelease was explicitly requested.

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
