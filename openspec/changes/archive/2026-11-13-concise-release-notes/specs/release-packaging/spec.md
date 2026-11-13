## MODIFIED Requirements

### Requirement: Published releases SHALL contain curated release information
The final GitHub Release description SHALL expand or replace auto-generated notes with a structured summary derived from the commits, diff, and completed OpenSpec changes since the previous tag. Unless the requester specifies another language arrangement, the summary SHALL be bilingual: the English summary first, followed by the corresponding Simplified Chinese version, and SHALL cover user-visible highlights and fixes, compatibility or migration information, available platform downloads, verification results, and a full comparison link. The final Release SHALL be a non-draft stable release unless a prerelease was explicitly requested.

Release notes SHALL default to a concise user-facing summary: one opening sentence and normally 3–5 combined highlights/fixes per language, using fewer items for small patches. Compatibility, downloads and verification SHALL use brief lines, and implementation history, benchmark tables, detailed test logs and release-operation records SHALL live in linked documentation. Required migration, first-launch instructions, signing/notarization status and unverified or skipped scope SHALL remain explicit. Extra detail SHALL be limited to what is necessary for safe upgrading or explicitly requested by the maintainer. AGENTS.md and the canonical release-note template SHALL reflect this concise default.

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
- **AND** confirms that the Windows NSIS installer, Windows portable `.zip`, macOS Apple Silicon DMG, Linux amd64 DEB, Linux x86_64 RPM, and Linux x86_64 AppImage are attached
- **AND** confirms that the curated notes and comparison link are present

#### Scenario: Routine release notes stay brief
- **WHEN** an operator writes notes for a future routine release
- **THEN** each language uses one summary sentence and normally 3–5 combined highlights/fixes, or fewer for a small patch
- **AND** compact compatibility, downloads and verification lines retain the required coverage and comparison link
- **AND** detailed implementation, measurements, test logs and release retries are linked instead of reproduced

#### Scenario: Essential upgrade information needs more space
- **WHEN** brevity would omit a required migration, first-launch instruction, signing/notarization status or unverified/skipped scope
- **THEN** the operator keeps that essential information even if the notes need extra detail
