## Context

The existing tag-triggered GitHub Actions workflow already performs native Windows, macOS, and Linux packaging and creates a GitHub Release. The missing layer is the operator workflow around it: version choice, synchronized metadata, validation, commit/tag conventions, CI monitoring, curated release notes, and final asset verification. The v0.1.5 release established a successful sequence that can now be captured in repository-owned documentation.

The process must work for both human maintainers and repository agents. It must also avoid treating a pushed tag or an auto-generated release page as completion before all installers and meaningful notes are present.

## Goals / Non-Goals

**Goals:**

- Make `docs/release-process.md` the canonical, executable release checklist.
- Make `AGENTS.md` direct future agents to that checklist and define safe defaults when a user asks only to "publish a new version."
- Keep every version-bearing file synchronized and make local and CI verification explicit.
- Require curated release notes and final asset verification before reporting success.

**Non-Goals:**

- Change the GitHub Actions build matrix or packaging formats.
- Add signing, notarization, auto-update, crates.io publication, or a new release automation dependency.
- Automate rollback by deleting or moving a public tag.

## Decisions

### Use a repository runbook plus a short agent rule

The detailed procedure will live in `docs/release-process.md`; `AGENTS.md` will contain the mandatory defaults and a link to the runbook. This keeps the durable instructions visible to both maintainers and agents without duplicating a long checklist in multiple places.

Alternative considered: record the process only in chat memory or `AGENTS.md`. Chat memory is not repository-owned, and a full checklist in `AGENTS.md` would obscure the project's more general development rules.

### Default an unspecified release to the next patch version

When the requester does not supply a version, the operator will take the highest stable `vMAJOR.MINOR.PATCH` tag and increment PATCH. An explicit requested version takes precedence after checking that neither the tag nor GitHub Release already exists. Prerelease and breaking-version decisions remain explicit user choices.

Alternative considered: always stop and ask for a version. That adds unnecessary friction to routine releases and does not match the completed v0.1.5 workflow.

### Treat version metadata as one atomic release change

The workspace package version and root package version in `Cargo.toml`, the package version in `packager.toml`, and the affected workspace package entries in `Cargo.lock` will move together. `cargo metadata --no-deps` verifies the resolved workspace versions before the release commit.

### Separate content verification from publication verification

Before publication, run `cargo test --workspace`, verify the diff, and create one `Release Markion vX.Y.Z` commit plus an annotated `vX.Y.Z` tag. After pushing `main` and the tag, monitor the tag workflow through completion, then verify the published Release, its four distributable assets, and the repository's final clean/synchronized state.

This two-stage gate prevents a successful local test from masking packaging failures and prevents a successful tag push from being reported as a completed release.

### Curate the auto-generated release page after CI publishes it

GitHub's generated notes remain a useful seed but are not the final release description. The operator will replace or expand them with a Simplified Chinese summary by default, unless the requester specifies another language. The notes will cover user-visible highlights and fixes, compatibility or migration information, available downloads, verification status, and a full comparison link. The description must be derived from the actual diff, commits, and completed OpenSpec changes since the previous tag.

### Prefer fix-forward over destructive tag rewriting

If publication fails after a tag is public, the operator will report the failed stage and fix forward or ask for direction. Public tags and releases will not be deleted, force-moved, or silently recreated without explicit authorization.

## Risks / Trade-offs

- [The documented version locations may change] → Search the repository for the previous version and verify resolved package versions with `cargo metadata` before committing.
- [A platform build can fail after the tag is public] → Monitor the tag workflow to completion and do not claim success while any required job or asset is missing.
- [Generated notes can omit direct commits] → Build curated notes from the full tag-to-HEAD diff, commit log, and completed OpenSpec artifacts rather than relying only on merged pull requests.
- [A duplicate main-branch workflow consumes CI time] → Accept the current behavior because the tag workflow is authoritative for release publication; changing workflow triggers is outside this change.
- [Unsigned installers trigger operating-system warnings] → Include the existing SmartScreen and Gatekeeper limitation in compatibility notes.
