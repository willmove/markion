# release-packaging Specification

## Purpose
Define how installable Markion releases are produced and distributed: a per-platform native build matrix (Windows/macOS/Linux) driven by GitHub Actions — required because `gpui`'s per-OS GPU backends cannot be cross-compiled — plus the `cargo-packager` installer formats (NSIS, `.app`/`.dmg`, `.deb`/`.AppImage`) and their unsigned-build limitations.
## Requirements
### Requirement: Per-platform native release builds via CI matrix
The project SHALL provide a GitHub Actions workflow that builds a release binary for each supported desktop platform by compiling natively on that platform's runner, because the `gpui` UI dependency uses a distinct native GPU backend per OS (DirectX on Windows, Vulkan/Wayland/X11 on Linux, Metal on macOS) that cannot be cross-compiled from a single host. The matrix SHALL cover Windows x86_64, Linux x86_64, and macOS arm64; each job SHALL produce the binary with `cargo build --release --target <triple>`.

#### Scenario: All three platforms build on every push
- **WHEN** a commit is pushed to `main` (or a pull request opens)
- **THEN** three CI jobs run in parallel — `ubuntu-22.04`, `macos-latest`, `windows-latest` — and each compiles the crate to a release binary for its target triple without cross-compilation

#### Scenario: Linux job installs the native dependencies gpui needs
- **WHEN** the Linux build job runs
- **THEN** it installs the system libraries `gpui` requires to link (clang, cmake, pkg-config, and the Wayland/X11/Vulkan/xkbcommon/fontconfig/glib/openssl/alsa development packages) before building

#### Scenario: Build caches keep repeat runs affordable
- **WHEN** a subsequent build job runs on the same target
- **THEN** the cargo registry, git dependencies, and `target/` are restored from cache so the build skips already-compiled crates

### Requirement: Each release build SHALL be packaged into a native installer
After a successful per-platform build, the workflow SHALL run `cargo-packager` (driven by `Packager.toml`) to wrap the release binary into the platform-appropriate distributable format(s): a Windows NSIS `.exe` installer (current-user install mode), a macOS `.app` bundle plus `.dmg` disk image, and a Linux `.deb` package plus `.AppImage`. The packager config SHALL specify the product name (`Markion`), bundle identifier (`dev.markion.app`), version, category, and generated platform icon files (`assets/markion.ico`, `assets/markion.icns`, and `assets/markion.png`).

#### Scenario: Windows job produces an NSIS installer
- **WHEN** the Windows build job packages its binary
- **THEN** it emits a single NSIS `.exe` setup file that installs for the current user (no admin elevation required), creates Start Menu / Desktop shortcuts, and registers an Add/Remove-Programs entry

#### Scenario: macOS job produces an app bundle and disk image
- **WHEN** the macOS build job packages its binary
- **THEN** it emits a `.app` bundle and a `.dmg` disk image, both arm64 (Apple Silicon); the app icon is `assets/markion.icns`

#### Scenario: Linux job produces a deb and an AppImage
- **WHEN** the Linux build job packages its binary
- **THEN** it emits a `.deb` package (amd64) and a portable `.AppImage`, both using the generated `assets/markion.png` icon and the `dev.markion.app` desktop entry identifier

### Requirement: Version tags SHALL publish a GitHub Release with all installers
The workflow SHALL include a release job that runs only when a `v*` tag is pushed, downloads all per-platform packaging artifacts, and attaches them to a GitHub Release with auto-generated release notes. Builds on non-tag refs (branch pushes, pull requests) SHALL produce downloadable CI artifacts but SHALL NOT publish a release.

#### Scenario: Pushing a version tag publishes installers
- **WHEN** a tag matching `v*` is pushed
- **THEN** a GitHub Release is created (or updated) for that tag, with every platform's installer attached as downloadable assets and changelog notes generated from commits since the previous tag

#### Scenario: Branch pushes do not publish
- **WHEN** a commit is pushed to `main` or a pull request is opened
- **THEN** the build jobs run and upload artifacts to the workflow run, but no GitHub Release is created

### Requirement: Builds are unsigned and documented as such
The release pipeline SHALL NOT code-sign the macOS or Windows installers (no paid code-signing certificate is provisioned). The project SHALL document that end users will see Gatekeeper (macOS) / SmartScreen (Windows) warnings on first launch and must bypass them manually. macOS builds target arm64 only; Intel Mac users SHALL run via Rosetta. No universal (arm64+x86_64) binary, no notarization, and no auto-update channel are provided.

#### Scenario: Unsigned macOS build warns the user
- **WHEN** a user opens the distributed `.app` on macOS for the first time
- **THEN** Gatekeeper reports an unidentified developer, and the user must right-click → Open (or strip the quarantine attribute) to launch it — this is documented behavior, not a defect

#### Scenario: Unsigned Windows build warns the user
- **WHEN** a user runs the distributed NSIS installer on Windows for the first time
- **THEN** SmartScreen shows a "Windows protected your PC" warning, and the user must choose "More info → Run anyway" — this is documented behavior, not a defect

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

