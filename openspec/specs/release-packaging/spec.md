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
The final GitHub Release description SHALL expand or replace auto-generated notes with a structured summary derived from the commits, diff, and completed OpenSpec changes since the previous tag. Unless the requester specifies another language arrangement, the summary SHALL be bilingual: the English summary first, followed by the corresponding Simplified Chinese version, and SHALL cover user-visible highlights and fixes, compatibility or migration information, available platform downloads, verification results, and a full comparison link. The final Release SHALL be a non-draft stable release unless a prerelease was explicitly requested.

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

### Requirement: Tagged releases SHALL be mirrored to Aliyun OSS
Upon successful completion of the per-platform `build` jobs for a `v*` tag, the release workflow SHALL run a `mirror-oss` job that downloads the per-platform packaging artifacts, computes a SHA-256 digest for each installer, generates a `manifest.json` describing the release, and uploads the Windows NSIS installer, macOS DMG, Linux DEB, Linux AppImage, `packager.toml`, `manifest.json`, and `sha256sums.txt` to a stable `${OSS_PREFIX}/latest/` path on the configured Aliyun OSS Bucket. The OSS endpoint, Bucket name, AccessKey ID, and AccessKey Secret SHALL be supplied from repository secrets and SHALL NOT appear in the repository, the workflow file, or any OpenSpec artifact. The `mirror-oss` job SHALL depend on the `build` jobs and SHALL NOT depend on the `Publish GitHub Release` job; a failure of either job SHALL NOT prevent the other from running. A failure of the `mirror-oss` job SHALL be treated as an incomplete release requiring correction, even if the GitHub Release has already been published. The mirror SHALL overwrite any previous `${OSS_PREFIX}/latest/` objects so that the URL is a stable pointer to the newest release; per-tag history is retained only on GitHub Releases, not on OSS. The mirrored installers SHALL be byte-for-byte copies of the GitHub Release assets and SHALL NOT be code-signed or otherwise modified by the mirror step.

#### Scenario: Tagged release mirrors installers, config, and manifest to OSS
- **WHEN** a `v*` tag is pushed and all three native `build` jobs succeed
- **THEN** the `mirror-oss` job runs, downloads the per-platform packaging artifacts, and uploads the Windows NSIS installer, macOS DMG, Linux DEB, Linux AppImage, `packager.toml`, `manifest.json`, and `sha256sums.txt` to `${OSS_PREFIX}/latest/` on the configured OSS Bucket
- **AND** each file's OSS object key preserves its original filename under the `latest/` prefix

#### Scenario: OSS credentials come from secrets, not the repository
- **WHEN** the `mirror-oss` job runs
- **THEN** the OSS endpoint, Bucket, AccessKey ID, and AccessKey Secret are read from repository secrets (`OSS_ENDPOINT`, `OSS_BUCKET`, `OSS_ACCESS_KEY_ID`, `OSS_ACCESS_KEY_SECRET`)
- **AND** the OSS prefix and public base URL are read from repository secrets (`OSS_PREFIX`, `OSS_PUBLIC_BASE`)
- **AND** no OSS credential appears in the repository, the workflow file, or the OpenSpec change

#### Scenario: Branch pushes and pull requests do not mirror to OSS
- **WHEN** a commit is pushed to `main` or a pull request is opened
- **THEN** the `build` jobs run and upload workflow artifacts, but the `mirror-oss` job does not run

#### Scenario: Mirror upload is independent of GitHub Release publication
- **WHEN** the `build` jobs succeed for a `v*` tag
- **THEN** the `mirror-oss` job and the `Publish GitHub Release` job both run, neither depending on the other
- **AND** a failure of one job does not prevent the other from running to completion

#### Scenario: Mirror failure marks the release incomplete
- **WHEN** the `mirror-oss` job fails for a `v*` tag
- **THEN** the operator does not report the release as complete and corrects or retries the mirror upload, even though the GitHub Release may already be published
- **AND** the public version tag is preserved unless explicit authorization is given for a destructive correction

#### Scenario: Manifest describes the mirrored release
- **WHEN** the `mirror-oss` job generates `manifest.json`
- **THEN** the manifest contains the release version (without the leading `v`), the tag name, an ISO-8601 publication timestamp, and a map from platform identifier (`windows-x86_64`, `macos-aarch64`, `linux-amd64`, `linux-appimage`) to the installer filename
- **AND** the manifest remains available as mirror metadata without being required for the initial in-app update-check verification

### Requirement: The app SHALL check GitHub's latest published Release for updates
The application SHALL expose a "Check for Updates…" action in the Help menu that, when invoked, fetches `https://api.github.com/repos/willmove/markion/releases/latest` via the existing in-app HTTP layer, parses the latest published Release, compares its `tag_name` (after the required leading `v`) against `env!("CARGO_PKG_VERSION")` using a semantic-version comparison, and surfaces the result through a modal dialog. When the Release version is newer than the running version, the dialog SHALL display the newer version number without rendering the raw asset URL and SHALL provide localized actionable buttons. On a Windows x86_64 tagged build containing the configured updater public key, the primary action SHALL download the update described by the published updater manifest, verify the cargo-packager Minisign signature before execution, launch the verified current-user NSIS installer in passive mode, and exit Markion only after the installer starts. Automatic installation SHALL NOT begin while any document has unsaved changes. On macOS, Linux, unsupported architectures, or builds without updater public-key material, the primary action SHALL open the matching GitHub asset URL in the system browser, or the Release page when no platform package mapping exists. When the Release version is equal to or older than the running version, the dialog SHALL report that the application is up to date. When the fetch, manifest, download, signature verification, or installer launch fails, or a supported-platform asset is missing, the application SHALL report the failure without crashing and SHALL retain a manual-download fallback. The application SHALL NOT download or install an update without an explicit user action. The application MAY additionally perform the release check on startup when the `check_for_updates_on_startup` preference is `true` (default `false`), in which case it SHALL be silent unless a newer version is found, and SHALL record the check timestamp in the `last_update_check` preference. Update work SHALL run off the main render path and SHALL NOT recompute any cached-per-version Markdown state.

#### Scenario: User invokes Check for Updates and a newer version is available
- **WHEN** the user chooses "Check for Updates…" on a Windows x86_64 tagged build, the published Release is newer, updater public-key material is present, and all documents are clean
- **THEN** a modal dialog shows the newer version and offers "Download and Install" and "Later" actions without displaying a truncated raw URL
- **AND** choosing "Download and Install" downloads and verifies the signed NSIS installer off the main render path
- **AND** Markion launches the verified installer in passive mode and exits only after the installer starts

#### Scenario: User chooses the download fallback
- **WHEN** a newer Release is available on macOS, Linux, an unsupported architecture, or a build without updater public-key material
- **THEN** the modal dialog shows the newer version and offers "Download Update" and "Later" actions without displaying a truncated raw URL
- **AND** choosing "Download Update" opens the matching platform asset, or the Release page when no package mapping exists, in the system browser
- **AND** no file is installed automatically

#### Scenario: Unsaved documents block automatic installation
- **WHEN** the user chooses "Download and Install" while any open document has unsaved changes
- **THEN** Markion does not download, execute, or install the update
- **AND** it asks the user to save their work and retry
- **AND** all open document state remains unchanged

#### Scenario: User invokes Check for Updates and is up to date
- **WHEN** the user chooses "Check for Updates…" and the latest published Release version is equal to or older than `CARGO_PKG_VERSION`
- **THEN** a modal dialog appears reporting that the application is up to date

#### Scenario: Update check fails without crashing
- **WHEN** the GitHub latest-release fetch fails, the response is not valid JSON, the tag cannot be parsed, or the matching supported-platform asset is missing
- **THEN** a modal dialog appears reporting that the update check failed
- **AND** the user can open the latest Release page through the manual-download fallback
- **AND** the application continues running normally

#### Scenario: Signed installation fails safely
- **WHEN** the updater manifest is invalid, the download fails, signature verification fails, or the installer cannot be launched
- **THEN** Markion reports that automatic update failed and does not execute an unverified payload
- **AND** the current installation and open documents remain usable
- **AND** the user can open the matching Release asset through the manual-download fallback

#### Scenario: Startup check is opt-in and silent unless newer
- **WHEN** the `check_for_updates_on_startup` preference is `true` and the application starts
- **THEN** the application performs the same GitHub latest-release fetch and version comparison as the menu action
- **AND** no dialog appears unless the GitHub Release version is newer than the running version
- **AND** the timestamp of the check is recorded in the `last_update_check` preference

#### Scenario: Update check does not alter cached Markdown state
- **WHEN** update discovery, download, verification, or installation preparation runs
- **THEN** it executes off the main render path and does not recompute any preview block, outline, stat, syntax-highlighting, or text-handle cache

### Requirement: Tagged releases SHALL publish authenticated Windows updater metadata
For every stable `v*` release, the publication pipeline SHALL create a cargo-packager Minisign signature for the Windows x86_64 NSIS installer and an updater manifest containing the release version, the signed installer's public download URL, its signature, and the `nsis` update format. The signature and updater manifest SHALL be attached to the GitHub Release and mirrored beside the installer under the Aliyun OSS `latest/` prefix. The signing private key and password SHALL come from repository secrets, the corresponding public key SHALL be embedded only in tagged application builds, and no private signing material SHALL be written to repository files or build artifacts. Updater signing SHALL be treated as payload authentication and SHALL NOT be represented as Windows Authenticode signing.

#### Scenario: Tagged Windows installer receives updater metadata
- **WHEN** a stable `v*` release packages the Windows x86_64 NSIS installer
- **THEN** the pipeline signs that exact installer with the configured updater private key
- **AND** publishes its `.sig` file and an updater-compatible `update.json` to both the GitHub Release and Aliyun OSS
- **AND** the manifest directs Windows x86_64 clients to the signed installer mirrored on OSS

#### Scenario: Signing secrets are unavailable
- **WHEN** a tagged release cannot read the updater private key, password, or matching public key from repository secrets
- **THEN** signed update metadata publication fails
- **AND** the release is not reported as complete
- **AND** no placeholder key or unsigned automatic-update manifest is published

#### Scenario: Non-tag builds do not publish updater metadata
- **WHEN** a branch push or pull request runs the native build matrix
- **THEN** application compilation remains valid without signing secrets
- **AND** no updater signature or public update manifest is published

