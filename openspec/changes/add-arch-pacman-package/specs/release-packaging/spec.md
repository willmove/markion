## MODIFIED Requirements

### Requirement: Each release build SHALL be packaged into a native installer
After a successful per-platform build, the workflow SHALL use `cargo-packager` (driven by `Packager.toml`) and the existing supplementary package builders to produce the platform-appropriate distributable formats: a Windows NSIS `.exe` installer (current-user install mode) plus a portable `.zip` archive, a macOS `.app` bundle plus `.dmg` disk image, and Linux `.deb`, `.rpm`, `.AppImage`, and pacman `.pkg.tar.zst` packages. The packager config SHALL specify the product name (`Markion`), bundle identifier (`dev.markion.app`), version, category, and generated platform icon files (`assets/markion.ico`, `assets/markion.icns`, and `assets/markion.png`). The Windows portable archive SHALL contain the application binary and the same bundled resource payload as the NSIS install layout under a single top-level folder so extraction does not scatter files. The pacman package SHALL contain the same Linux application and resource payload as the DEB/RPM packages, use the `x86_64` architecture, declare Arch-native runtime dependencies, and be named `markion-<version>-<release>-x86_64.pkg.tar.zst` using the conventional pacman name, version, release and architecture fields.

#### Scenario: Windows job produces an NSIS installer
- **WHEN** the Windows build job packages its binary
- **THEN** it emits a single NSIS `.exe` setup file that installs for the current user (no admin elevation required), creates Start Menu / Desktop shortcuts, and registers an Add/Remove-Programs entry
- **AND** it emits `markion_<version>_x64-portable.zip` containing a top-level portable folder with the application executable and the same resources the installer would place beside it

#### Scenario: macOS job produces an app bundle and disk image
- **WHEN** the macOS build job packages its binary
- **THEN** it emits a `.app` bundle and a `.dmg` disk image, both arm64 (Apple Silicon); the app icon is `assets/markion.icns`

#### Scenario: Linux job produces a deb and an AppImage
- **WHEN** the Linux build job packages its binary and validates the pacman output
- **THEN** it emits a `.deb` package (amd64), an `.rpm` package (x86_64), a portable `.AppImage`, and a `.pkg.tar.zst` package (x86_64)
- **AND** every package uses the generated `assets/markion.png` icon and the `dev.markion.app` desktop entry identifier where the format supports a desktop entry
- **AND** the pacman package can be inspected and installed by Arch Linux tooling without converting a foreign package at install time

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
- **AND** confirms that the Windows NSIS installer, Windows portable `.zip`, macOS Apple Silicon DMG, Linux amd64 DEB, Linux x86_64 RPM, Linux x86_64 AppImage, and Linux x86_64 pacman `.pkg.tar.zst` are attached
- **AND** confirms that the curated notes and comparison link are present

#### Scenario: Routine release notes stay brief
- **WHEN** an operator writes notes for a future routine release
- **THEN** each language uses one summary sentence and normally 3–5 combined highlights/fixes, or fewer for a small patch
- **AND** compact compatibility, downloads and verification lines retain the required coverage and comparison link
- **AND** detailed implementation, measurements, test logs and release retries are linked instead of reproduced

#### Scenario: Essential upgrade information needs more space
- **WHEN** brevity would omit a required migration, first-launch instruction, signing/notarization status or unverified/skipped scope
- **THEN** the operator keeps that essential information even if the notes need extra detail

### Requirement: Tagged releases SHALL be mirrored to Aliyun OSS
Upon successful completion of the per-platform `build` jobs, including pacman packaging and installation verification, for a `v*` tag, the release workflow SHALL run a `mirror-oss` job that downloads the per-platform packaging artifacts, computes a SHA-256 digest for each installer package, generates a `manifest.json` describing the release, and uploads the Windows NSIS installer, Windows portable `.zip`, macOS DMGs, Linux DEB, Linux RPM, Linux AppImage, Linux pacman `.pkg.tar.zst`, `packager.toml`, `manifest.json`, and `sha256sums.txt` to a stable `${OSS_PREFIX}/latest/` path on the configured Aliyun OSS Bucket. The OSS endpoint, Bucket name, AccessKey ID, and AccessKey Secret SHALL come from repository secrets and SHALL NOT appear in the repository, the workflow file, or any OpenSpec artifact. The `mirror-oss` job SHALL depend on the native build jobs, including their pacman packaging and installation verification, and SHALL NOT depend on the `Publish GitHub Release` job; a failure of either job SHALL NOT prevent the other from running. A failure of the `mirror-oss` job SHALL be treated as an incomplete release requiring correction, even if the GitHub Release has already been published. The mirror SHALL overwrite any previous `${OSS_PREFIX}/latest/` objects so that the URL is a stable pointer to the newest release; per-tag history is retained only on GitHub Releases, not on OSS. The mirrored installers SHALL be byte-for-byte copies of the GitHub Release assets and SHALL NOT be code-signed or otherwise modified by the mirror step.

#### Scenario: Tagged release mirrors installers, config, and manifest to OSS
- **WHEN** a `v*` tag is pushed and all native build jobs, including pacman packaging and installation verification, succeed
- **THEN** the `mirror-oss` job runs, downloads the packaging artifacts, and uploads the Windows NSIS installer, Windows portable `.zip`, macOS DMGs, Linux DEB, Linux RPM, Linux AppImage, Linux pacman `.pkg.tar.zst`, `packager.toml`, `manifest.json`, and `sha256sums.txt` to `${OSS_PREFIX}/latest/` on the configured OSS Bucket
- **AND** each file's OSS object key preserves its original filename under the `latest/` prefix

#### Scenario: OSS credentials come from secrets, not the repository
- **WHEN** the `mirror-oss` job runs
- **THEN** the OSS endpoint, Bucket, AccessKey ID, and AccessKey Secret are read from repository secrets (`OSS_ENDPOINT`, `OSS_BUCKET`, `OSS_ACCESS_KEY_ID`, `OSS_ACCESS_KEY_SECRET`)
- **AND** the OSS prefix and public base URL are read from repository secrets (`OSS_PREFIX`, `OSS_PUBLIC_BASE`)
- **AND** no OSS credential appears in the repository, the workflow file, or the OpenSpec change

#### Scenario: Branch pushes and pull requests do not mirror to OSS
- **WHEN** a commit is pushed to `main` or a pull request is opened
- **THEN** the build and package jobs run and upload workflow artifacts, but the `mirror-oss` job does not run

#### Scenario: Mirror upload is independent of GitHub Release publication
- **WHEN** the build and package jobs succeed for a `v*` tag
- **THEN** the `mirror-oss` job and the `Publish GitHub Release` job both run, neither depending on the other
- **AND** a failure of one job does not prevent the other from running to completion

#### Scenario: Mirror failure marks the release incomplete
- **WHEN** the `mirror-oss` job fails for a `v*` tag
- **THEN** the operator does not report the release as complete and corrects or retries the mirror upload, even though the GitHub Release may already be published
- **AND** the public version tag is preserved unless explicit authorization is given for a destructive correction

#### Scenario: Manifest describes the mirrored release
- **WHEN** the `mirror-oss` job generates `manifest.json`
- **THEN** the manifest contains the release version (without the leading `v`), the tag name, an ISO-8601 publication timestamp, and a map from platform identifier (`windows-x86_64`, `windows-x86_64-portable`, `macos-aarch64`, `macos-x86_64`, `linux-amd64`, `linux-rpm`, `linux-appimage`, `linux-arch-x86_64`) to the installer package filename
- **AND** the manifest remains available as mirror metadata without being required for the initial in-app update-check verification

#### Scenario: Mirror repair includes the pacman package
- **WHEN** a maintainer repairs the OSS mirror for an existing release that contains a pacman package
- **THEN** the repair downloads, uploads, checksums and verifies that package together with all existing packages and metadata

### Requirement: The app SHALL check GitHub's latest published Release for updates

The application SHALL expose a localized Help → Check for Updates action and SHALL automatically check `https://api.github.com/repos/willmove/markion/releases/latest` at most once per five-hour interval during normal operation, including an overdue check shortly after startup. It SHALL compare the required `v`-prefixed release tag with the running semantic version, persist each check attempt as an RFC 3339 `last_update_check`, tolerate absent, invalid or future timestamps, and prevent concurrent checks, downloads or installations. The legacy `check_for_updates_on_startup` preference SHALL remain readable; when true it SHALL request a startup check even before the normal interval expires. Manual checks SHALL bypass the interval.

On Windows x86_64 with an embedded updater public key and on keyed Linux x86_64 AppImage launches, a newer release SHALL be downloaded and authenticated automatically when `automatically_download_updates` is true (default true). Otherwise the application SHALL ask before downloading. The signed updater SHALL try the Aliyun OSS manifest first, then GitHub, and use the successful manifest's package URL and signature. The manifest version SHALL match the discovered GitHub release. A downloaded package SHALL be verified with the configured Minisign key before becoming ready. Downloading SHALL NOT install, replace application files, quit or restart Markion. A ready package SHALL remain available for the current session without repeated downloading and SHALL be checked for tampering before installation.

When ready, the application SHALL offer localized Restart and Update and Later actions. Later SHALL preserve the ready package; Help → Check for Updates SHALL offer it again. Installation SHALL require explicit user confirmation and SHALL recheck for unsaved documents immediately before handoff. With clean documents, Windows SHALL launch the verified passive current-user NSIS installer with relaunch requested, persist session/layout state and quit only after successful handoff. Linux AppImage SHALL replace the running AppImage safely, relaunch it and quit only after successful replacement and relaunch. Failed AppImage replacement SHALL leave the old AppImage runnable.

On macOS, non-AppImage Linux installations, unsupported architectures or keyless builds, a newer-release notification SHALL retain the matching platform asset browser-download action (or Release-page fallback), without automatic browser opening or unsigned self-installation. Manual checks SHALL report current-version and error results with an actionable browser fallback. Routine background current-version and discovery failures SHALL remain silent; automatic download failures SHALL retain manual recovery without repeated modal interruption. Network, verification and package I/O SHALL run off the render path and SHALL NOT recompute version-cached Markdown state.

On Linux x86_64 browser-download paths, an AppImage launch SHALL retain the AppImage asset. For other launches, the offered asset SHALL follow the distribution family read from `/etc/os-release`: if `ID` or a space-separated `ID_LIKE` token equals `arch`, offer the pacman `.pkg.tar.zst` package; otherwise offer `.deb`, including absent or unparseable os-release. The pacman download SHALL NOT be represented as an authenticated in-app self-update or installed automatically.

#### Scenario: User invokes Check for Updates and a newer version is available
- **WHEN** a newer release is discovered on a supported signed-update installation with automatic downloads enabled
- **THEN** Markion downloads and verifies it without asking first, including while documents are dirty
- **AND** no installer or restart begins before explicit confirmation

#### Scenario: User chooses the download fallback
- **WHEN** a newer release is found on a platform without authenticated self-install support
- **THEN** Markion displays the version and offers Download Update and Later without a raw asset URL
- **AND** only selecting Download Update opens the platform asset or Release page in the browser

#### Scenario: Unsaved documents block automatic installation
- **WHEN** the user confirms Restart and Update with any dirty document
- **THEN** Markion asks the user to save and retry, retains the ready update and leaves all documents unchanged

#### Scenario: User invokes Check for Updates and is up to date
- **WHEN** a manual check finds an equal or older release
- **THEN** Markion displays the localized up-to-date dialog

#### Scenario: Update check fails without crashing
- **WHEN** a manual check cannot fetch or parse the release, its version or matching asset
- **THEN** Markion reports a recoverable error and offers the Release-page download fallback

#### Scenario: Signed installation fails safely
- **WHEN** manifest versions disagree, signature verification fails, the staged package changes, or installation cannot start
- **THEN** no unverified package is executed and Markion remains usable with a manual-download fallback

#### Scenario: Startup check is opt-in and silent unless newer
- **WHEN** Markion restarts less than five hours after a check and the legacy startup override is false
- **THEN** automatic discovery waits for the remaining interval
- **AND** an invalid or future timestamp cannot suppress checks indefinitely

#### Scenario: Update check does not alter cached Markdown state
- **WHEN** discovery, downloading, verification or installation preparation runs
- **THEN** preview blocks, outline, stats, syntax highlighting and cached text handles retain their per-version behavior

#### Scenario: Manifest fetch falls back from OSS to GitHub
- **WHEN** OSS cannot supply a usable manifest
- **THEN** the updater tries the GitHub manifest
- **AND** the package URL and signature come from the manifest that succeeded

#### Scenario: A successful manifest is used end to end on one host
- **WHEN** a manifest endpoint succeeds
- **THEN** its package URL and signature are used together without switching to a failed host

#### Scenario: AppImage install updates itself in place
- **WHEN** the user confirms a ready AppImage update with clean documents
- **THEN** Markion replaces the AppImage at its original path, relaunches it and quits the current instance
- **AND** a replacement failure leaves the old AppImage runnable and a relaunch failure keeps the current process running with manual-restart guidance

#### Scenario: Overdue startup and periodic checks
- **WHEN** no valid recent check exists or the five-hour interval expires while Markion is open
- **THEN** a background check starts and its timestamp is persisted
- **AND** no second check or download overlaps an active update operation

#### Scenario: Automatic downloads disabled
- **WHEN** a newer release is found with automatic downloads disabled
- **THEN** Markion offers a download action and waits for the user to select it
- **AND** changing the preference before discovery completes is honored

#### Scenario: Ready update postponed and resumed
- **WHEN** the user chooses Later after a verified download
- **THEN** editing continues and the ready package remains available for this session
- **AND** the next manual check offers Restart and Update without downloading again

#### Scenario: Windows restart handoff
- **WHEN** the user confirms a ready Windows update and all documents are clean
- **THEN** the authenticated NSIS installer starts in passive mode with relaunch requested
- **AND** Markion persists its session/layout and quits after successful handoff

#### Scenario: Manual current-version and error reporting
- **WHEN** the user invokes Check for Updates and the release is not newer or discovery fails
- **THEN** Markion reports up-to-date status or a recoverable failure with manual-download fallback
- **AND** routine background checks for these same outcomes do not display modal dialogs

#### Scenario: Linux download follows the launch format and distribution family
- **WHEN** a newer release is offered through the Linux x86_64 browser-download path
- **THEN** an AppImage launch is offered the AppImage asset regardless of distribution
- **AND** a non-AppImage pacman-based system whose os-release ID or ID_LIKE contains arch is offered the pacman `.pkg.tar.zst` asset
- **AND** every other non-AppImage Linux system, including missing or unparseable os-release, is offered the DEB asset
- **AND** the browser opens only after the user chooses Download Update

### Requirement: Packaged local WeChat publishing workspace
Every supported Markion native package SHALL include the complete pinned MarkNice publishing workspace, its locally hosted third-party runtime assets and fonts, a machine-readable provenance manifest, and applicable license notices. Release construction and verification SHALL require neither a sibling MarkNice checkout nor Node.js, SHALL verify that the bundled application shell has no remote runtime dependency, and SHALL fail before publication when required workspace assets are absent, unlisted, or inconsistent with their manifest.

#### Scenario: Native packages contain the workspace
- **WHEN** the Windows NSIS installer, Windows portable `.zip`, macOS Apple Silicon application/DMG, Linux amd64 DEB, Linux x86_64 RPM, Linux x86_64 AppImage, or Linux x86_64 pacman `.pkg.tar.zst` is assembled
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

## ADDED Requirements

### Requirement: Pacman packages SHALL pass an Arch userspace installation gate
Before a pacman package is published, the release workflow SHALL inspect its metadata and install it with `pacman` in an Arch Linux userspace. The installed executable, desktop entry, icon, examples and MarkNice workspace SHALL match the intended Linux payload. A packaging, dependency-metadata, installation or resource verification failure SHALL block GitHub and OSS publication. This installation gate SHALL NOT be described as an Omarchy desktop interaction test or as package-manager signing.

#### Scenario: Valid package installs on Arch Linux
- **WHEN** the release workflow builds the Linux x86_64 pacman package
- **THEN** Arch Linux tooling validates its name, version, release, architecture and runtime dependencies
- **AND** `pacman` installs the package and the expected executable and resources are present

#### Scenario: Broken pacman package blocks publication
- **WHEN** pacman metadata, installation, architecture or packaged-resource verification fails
- **THEN** the workflow fails and neither GitHub Release publication nor OSS mirroring treats the package as ready

#### Scenario: Omarchy compatibility scope is stated accurately
- **WHEN** release notes or verification records describe Omarchy support
- **THEN** they identify the package as compatible through its Arch package format and dependencies
- **AND** they do not claim a native Omarchy desktop acceptance pass unless that environment was actually tested
