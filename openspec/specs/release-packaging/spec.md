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
After a successful per-platform build, the workflow SHALL run `cargo-packager` (driven by `Packager.toml`) to wrap the release binary into the platform-appropriate distributable format(s): a Windows NSIS `.exe` installer (current-user install mode) plus a portable `.zip` archive, a macOS `.app` bundle plus `.dmg` disk image, and a Linux `.deb` package, `.rpm` package, plus `.AppImage`. The packager config SHALL specify the product name (`Markion`), bundle identifier (`dev.markion.app`), version, category, and generated platform icon files (`assets/markion.ico`, `assets/markion.icns`, and `assets/markion.png`). The Windows portable archive SHALL contain the application binary and the same bundled resource payload as the NSIS install layout under a single top-level folder so extraction does not scatter files.

#### Scenario: Windows job produces an NSIS installer
- **WHEN** the Windows build job packages its binary
- **THEN** it emits a single NSIS `.exe` setup file that installs for the current user (no admin elevation required), creates Start Menu / Desktop shortcuts, and registers an Add/Remove-Programs entry
- **AND** it emits `markion_<version>_x64-portable.zip` containing a top-level portable folder with the application executable and the same resources the installer would place beside it

#### Scenario: macOS job produces an app bundle and disk image
- **WHEN** the macOS build job packages its binary
- **THEN** it emits a `.app` bundle and a `.dmg` disk image, both arm64 (Apple Silicon); the app icon is `assets/markion.icns`

#### Scenario: Linux job produces a deb and an AppImage
- **WHEN** the Linux build job packages its binary
- **THEN** it emits a `.deb` package (amd64), an `.rpm` package (x86_64), and a portable `.AppImage`, all using the generated `assets/markion.png` icon and the `dev.markion.app` desktop entry identifier where the format supports a desktop entry

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

### Requirement: Tagged releases SHALL be mirrored to Aliyun OSS
Upon successful completion of the per-platform `build` jobs for a `v*` tag, the release workflow SHALL run a `mirror-oss` job that downloads the per-platform packaging artifacts, computes a SHA-256 digest for each installer package, generates a `manifest.json` describing the release, and uploads the Windows NSIS installer, Windows portable `.zip`, macOS DMG, Linux DEB, Linux RPM, Linux AppImage, `packager.toml`, `manifest.json`, and `sha256sums.txt` to a stable `${OSS_PREFIX}/latest/` path on the configured Aliyun OSS Bucket. The OSS endpoint, Bucket name, AccessKey ID, and AccessKey Secret SHALL come from repository secrets and SHALL NOT appear in the repository, the workflow file, or any OpenSpec artifact. The `mirror-oss` job SHALL depend on the `build` jobs and SHALL NOT depend on the `Publish GitHub Release` job; a failure of either job SHALL NOT prevent the other from running. A failure of the `mirror-oss` job SHALL be treated as an incomplete release requiring correction, even if the GitHub Release has already been published. The mirror SHALL overwrite any previous `${OSS_PREFIX}/latest/` objects so that the URL is a stable pointer to the newest release; per-tag history is retained only on GitHub Releases, not on OSS. The mirrored installers SHALL be byte-for-byte copies of the GitHub Release assets and SHALL NOT be code-signed or otherwise modified by the mirror step.

#### Scenario: Tagged release mirrors installers, config, and manifest to OSS
- **WHEN** a `v*` tag is pushed and all three native `build` jobs succeed
- **THEN** the `mirror-oss` job runs, downloads the per-platform packaging artifacts, and uploads the Windows NSIS installer, Windows portable `.zip`, macOS DMG, Linux DEB, Linux RPM, Linux AppImage, `packager.toml`, `manifest.json`, and `sha256sums.txt` to `${OSS_PREFIX}/latest/` on the configured OSS Bucket
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
- **THEN** the manifest contains the release version (without the leading `v`), the tag name, an ISO-8601 publication timestamp, and a map from platform identifier (`windows-x86_64`, `windows-x86_64-portable`, `macos-aarch64`, `linux-amd64`, `linux-rpm`, `linux-appimage`) to the installer package filename
- **AND** the manifest remains available as mirror metadata without being required for the initial in-app update-check verification

### Requirement: The app SHALL check GitHub's latest published Release for updates
The signed update flow SHALL try the Aliyun OSS manifest first and the GitHub Release manifest second, falling back after an unsuccessful endpoint. The installer URL and signature SHALL come from the manifest that succeeded.

The application SHALL expose a "Check for Updates…" action in the Help menu that, when invoked, fetches `https://api.github.com/repos/willmove/markion/releases/latest` via the existing in-app HTTP layer, parses the latest published Release, compares its `tag_name` (after the required leading `v`) against `env!("CARGO_PKG_VERSION")` using a semantic-version comparison, and surfaces the result through a modal dialog. When the Release version is newer than the running version, the dialog SHALL display the newer version number without rendering the raw asset URL and SHALL provide localized actionable buttons. On a Windows x86_64 tagged build containing the configured updater public key, the primary action SHALL download the update described by the published updater manifest, verify the cargo-packager Minisign signature before execution, launch the verified current-user NSIS installer in passive mode, and exit Markion only after the installer starts. On a Linux x86_64 tagged build containing the configured updater public key and launched as an AppImage (a non-empty `APPIMAGE` environment variable set by the AppImage runtime), the primary action SHALL download the update described by the published updater manifest, verify the cargo-packager Minisign signature of the AppImage payload before installation, replace the running AppImage file in place, relaunch the updated AppImage, and exit the current instance only after the replacement succeeds. Automatic installation SHALL NOT begin while any document has unsaved changes. On macOS, Linux installs that are not running as an AppImage (deb/rpm or extracted AppDir), unsupported architectures, or builds without updater public-key material, the primary action SHALL open the matching GitHub asset URL in the system browser, or the Release page when no platform package mapping exists. When the Release version is equal to or older than the running version, the dialog SHALL report that the application is up to date. When the fetch, manifest, download, signature verification, installer launch, AppImage replacement, or relaunch fails, or a supported-platform asset is missing, the application SHALL report the failure without crashing and SHALL retain a manual-download fallback; a failed AppImage replacement SHALL leave the previously installed AppImage in place and runnable. The application SHALL NOT download or install an update without an explicit user action. The application MAY additionally perform the release check on startup when the `check_for_updates_on_startup` preference is `true` (default `false`), in which case it SHALL be silent unless a newer version is found, and SHALL record the check timestamp in the `last_update_check` preference. Update work SHALL run off the main render path and SHALL NOT recompute any cached-per-version Markdown state.

#### Scenario: User invokes Check for Updates and a newer version is available
- **WHEN** the user chooses "Check for Updates…" on a Windows x86_64 tagged build, the published Release is newer, updater public-key material is present, and all documents are clean
- **THEN** a modal dialog shows the newer version and offers "Download and Install" and "Later" actions without displaying a truncated raw URL
- **AND** choosing "Download and Install" downloads and verifies the signed NSIS installer off the main render path
- **AND** Markion launches the verified installer in passive mode and exits only after the installer starts

#### Scenario: User chooses the download fallback
- **WHEN** a newer Release is available on macOS, on a Linux install that is not running as an AppImage (deb/rpm or extracted AppDir), on an unsupported architecture, or on a build without updater public-key material
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
- **WHEN** the updater manifest is invalid, the download fails, signature verification fails, the installer cannot be launched, or the AppImage replacement or relaunch fails
- **THEN** Markion reports that automatic update failed and does not execute an unverified payload
- **AND** the current installation and open documents remain usable — on Linux the previously installed AppImage is restored to its original path and stays runnable
- **AND** the user can open the matching Release asset through the manual-download fallback

#### Scenario: Startup check is opt-in and silent unless newer
- **WHEN** the `check_for_updates_on_startup` preference is `true` and the application starts
- **THEN** the application performs the same GitHub latest-release fetch and version comparison as the menu action
- **AND** no dialog appears unless the GitHub Release version is newer than the running version
- **AND** the timestamp of the check is recorded in the `last_update_check` preference

#### Scenario: Update check does not alter cached Markdown state
- **WHEN** update discovery, download, verification, or installation preparation runs
- **THEN** it executes off the main render path and does not recompute any preview block, outline, stat, syntax-highlighting, or text-handle cache

#### Scenario: Manifest fetch falls back from OSS to GitHub
- **WHEN** the signed updater cannot reach the OSS manifest endpoint or it responds without a success status
- **THEN** the updater fetches the manifest from the GitHub Release asset endpoint
- **AND** the installer download and signature verification use the installer URL and signature published in the GitHub manifest

#### Scenario: A successful manifest is used end to end on one host
- **WHEN** an updater manifest endpoint returns a usable manifest
- **THEN** the installer is downloaded from the URL named by that manifest, which is hosted on the same distribution channel as the manifest
- **AND** no step of the signed update crosses back to a host that already failed

#### Scenario: AppImage install updates itself in place
- **WHEN** the user chooses "Check for Updates…" inside an AppImage-launched Linux x86_64 tagged build, the published Release is newer, updater public-key material is present, and all documents are clean
- **THEN** the dialog offers "Download and Install" and "Later" actions
- **AND** choosing "Download and Install" downloads and verifies the signed AppImage off the main render path
- **AND** Markion replaces the running AppImage file at its original path, relaunches the updated AppImage, and quits the current instance
- **AND** the whole operation requires no privilege escalation and no system package manager

### Requirement: Tagged releases SHALL publish authenticated Windows and Linux updater metadata
For every stable `v*` release, the publication pipeline SHALL create a cargo-packager Minisign signature for the Windows x86_64 NSIS installer and for the Linux x86_64 AppImage, and SHALL publish an updater manifest containing the release version and, per supported signed-update platform (`windows-x86_64`, `linux-x86_64`), the signed package's public download URL, its signature, and its update format (`nsis`, `appimage`). The signatures and updater manifest SHALL be attached to the GitHub Release and mirrored beside the installers under the Aliyun OSS `latest/` prefix, with each channel's manifest variant naming the installers hosted on that channel so the manifest, installer, and signature stay self-consistent per host. The signing private key and password SHALL come from repository secrets, the corresponding public key SHALL be embedded only in tagged Windows and Linux application builds, and no private signing material SHALL be written to repository files or build artifacts. Updater signing SHALL be treated as payload authentication and SHALL NOT be represented as Windows Authenticode signing or Linux package-manager signing.

#### Scenario: Tagged Windows installer receives updater metadata
- **WHEN** a stable `v*` release packages the Windows x86_64 NSIS installer
- **THEN** the pipeline signs that exact installer with the configured updater private key
- **AND** publishes its `.sig` file and an updater-compatible `update.json` to both the GitHub Release and Aliyun OSS
- **AND** each channel's manifest directs Windows x86_64 clients to the signed installer on that same distribution host

#### Scenario: Signing secrets are unavailable
- **WHEN** a tagged release cannot read the updater private key, password, or matching public key from repository secrets
- **THEN** signed update metadata publication fails
- **AND** the release is not reported as complete
- **AND** no placeholder key or unsigned automatic-update manifest is published

#### Scenario: Non-tag builds do not publish updater metadata
- **WHEN** a branch push or pull request runs the native build matrix
- **THEN** application compilation remains valid without signing secrets
- **AND** no updater signature or public update manifest is published

#### Scenario: Tagged installers receive updater metadata
- **WHEN** a stable `v*` release packages the Windows x86_64 NSIS installer and the Linux x86_64 AppImage
- **THEN** the pipeline signs each exact package with the configured updater private key
- **AND** publishes both `.sig` files and an updater-compatible `update.json` carrying both platform entries to the GitHub Release and Aliyun OSS
- **AND** the GitHub manifest variant directs clients to the GitHub-hosted packages and the OSS variant to the OSS-mirrored packages, with identical signatures in both

### Requirement: Packaged local WeChat publishing workspace
Every supported Markion native package SHALL include the complete pinned MarkNice publishing workspace, its locally hosted third-party runtime assets and fonts, a machine-readable provenance manifest, and applicable license notices. Release construction and verification SHALL require neither a sibling MarkNice checkout nor Node.js, SHALL verify that the bundled application shell has no remote runtime dependency, and SHALL fail before publication when required workspace assets are absent, unlisted, or inconsistent with their manifest.

#### Scenario: Native packages contain the workspace
- **WHEN** the Windows NSIS installer, Windows portable `.zip`, macOS Apple Silicon application/DMG, Linux amd64 DEB, Linux x86_64 RPM, or Linux x86_64 AppImage is assembled
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

### Requirement: Linux AppImage SHALL start without a host Vulkan driver
The Linux x86_64 AppImage SHALL contain a software Vulkan implementation and a Vulkan loader, built for the same C-library baseline as the application. When the host Vulkan stack cannot initialize Markion's renderer, the AppImage SHALL start on the bundled software implementation and show a usable main window without any user configuration or extra system packages.

#### Scenario: Host without any Vulkan driver opens a window
- **WHEN** the AppImage is launched on an X11 desktop (including Xvfb under `firejail --appimage`) that has no Vulkan driver installed and no GPU
- **THEN** the main Markion window appears and renders its content
- **AND** the diagnostic log records that the bundled software Vulkan driver is in use

#### Scenario: Host without a Vulkan loader still starts
- **WHEN** the AppImage is launched on a host that has neither a Vulkan loader nor a Vulkan driver
- **THEN** Markion starts on the bundled loader and software driver and shows its main window

#### Scenario: Bundled stack does not raise the C-library baseline
- **WHEN** the release pipeline assembles the AppImage
- **THEN** no bundled software-Vulkan file requires a newer glibc symbol version than the application binary itself

### Requirement: Hardware Vulkan SHALL stay preferred over the bundled software driver
The AppImage SHALL initialize the host Vulkan stack first and SHALL load the bundled software Vulkan stack only after host initialization has failed, at most once per launch. Users SHALL be able to force the bundled stack or disable the fallback through a documented environment variable.

#### Scenario: Working GPU driver keeps hardware rendering
- **WHEN** the AppImage is launched on a host whose Vulkan driver initializes successfully
- **THEN** Markion renders through the host driver and does not load the bundled software stack

#### Scenario: User forces or disables the software fallback
- **WHEN** the AppImage is launched with `MARKION_SOFTWARE_VULKAN=1`
- **THEN** Markion uses the bundled software stack without first trying the host driver
- **AND** when launched with `MARKION_SOFTWARE_VULKAN=0`, a host initialization failure is reported instead of falling back

#### Scenario: Fallback failure is reported, not retried forever
- **WHEN** renderer initialization also fails on the bundled software stack
- **THEN** Markion reports the renderer failure with remediation guidance and exits with a non-zero status without restarting again

### Requirement: Release pipeline SHALL verify that the AppImage opens a window without a GPU
Before a Linux AppImage is published, the release workflow SHALL launch it the way the AppImage catalog does — Xvfb with a window manager, `firejail --appimage`, on ubuntu-22.04 with no Vulkan driver installed — and SHALL fail unless a visible Markion window appears while the process is still running. The AppImage SHALL also keep its world-readable permissions and its updater signature SHALL be computed over the final file that contains the bundled stack.

#### Scenario: Catalog-equivalent launch succeeds
- **WHEN** the Linux build job has produced the AppImage
- **THEN** the smoke job finds a visible Markion window and a non-blank screenshot is uploaded as a workflow artifact

#### Scenario: Regression blocks publication
- **WHEN** the launched AppImage exits early or shows no visible window
- **THEN** the smoke job fails and no GitHub Release is published for that run

#### Scenario: Bundling keeps permissions and signatures valid
- **WHEN** the software Vulkan stack is added to the AppImage
- **THEN** AppRun, the application binary and every directory remain world-executable and every file world-readable
- **AND** the tagged-release `.sig` file and `update.json` entry are produced from that final AppImage

### Requirement: Updater manifest variants SHALL name their own distribution host
For every stable `v*` release, the publication pipeline SHALL produce two updater manifests for the same signed Windows installer: the manifest attached to the GitHub Release SHALL name the GitHub asset URL as the installer download URL, and the manifest mirrored to Aliyun OSS SHALL name the OSS object URL. Both variants SHALL carry the same release `version`, the same `format`, and byte-identical installer `signature` content, because the signed installer is byte-for-byte identical on both hosts. A variant SHALL NOT be published to the other host: the pipeline and the release procedure SHALL verify that each published manifest's installer URL names its own distribution host and that both variants report the tag version.

#### Scenario: GitHub Release manifest names the GitHub asset
- **WHEN** the `prepare-update` job publishes updater metadata for a stable `v*` release
- **THEN** the `update.json` attached to the GitHub Release directs Windows x86_64 clients to the NSIS installer asset on the GitHub Release
- **AND** its signature content matches the `.sig` published beside that installer

#### Scenario: OSS manifest names the OSS object
- **WHEN** the `mirror-oss` job uploads updater metadata for a stable `v*` release
- **THEN** the mirrored `update.json` directs Windows x86_64 clients to the NSIS installer under the OSS `latest/` prefix
- **AND** its signature content is identical to the GitHub manifest's signature content

#### Scenario: Variant swap is detected
- **WHEN** either publication target verifies its updater manifest
- **THEN** it confirms the manifest version equals the tag version and the installer URL names that target's own host
- **AND** a mismatch fails the release verification instead of being published
