## RENAMED Requirements

### FROM: Builds are unsigned and documented as such
### TO: macOS installers SHALL be ad-hoc signed; Windows SHALL remain unsigned with documented bypasses

## MODIFIED Requirements

### Requirement: Per-platform native release builds via CI matrix
The project SHALL provide a GitHub Actions workflow that builds a release binary for each supported desktop platform: Windows x86_64, Linux x86_64, and macOS arm64 compile natively on that platform's runner, because the `gpui` UI dependency uses a distinct native GPU backend per OS (DirectX on Windows, Vulkan/Wayland/X11 on Linux, Metal on macOS) that cannot be cross-compiled from a different OS. The matrix SHALL additionally cover macOS x86_64 by cross-compiling `--target x86_64-apple-darwin` on the arm64 macOS runner — same-OS cross-compilation against the same Metal SDK is supported and requires no Intel runner.

#### Scenario: All matrix jobs build on every push
- **WHEN** a commit is pushed to `main` (or a pull request opens)
- **THEN** four CI jobs run in parallel — `ubuntu-22.04`, `macos-latest` (arm64 native), `macos-latest` (cross-compiling `x86_64-apple-darwin`), and `windows-latest` — and each compiles the crate to a release binary for its target triple

#### Scenario: Linux job installs the native dependencies gpui needs
- **WHEN** the Linux build job runs
- **THEN** it installs the system libraries `gpui` requires to link (clang, cmake, pkg-config, and the Wayland/X11/Vulkan/xkbcommon/fontconfig/glib/openssl/alsa development packages) before building

#### Scenario: Build caches keep repeat runs affordable
- **WHEN** a subsequent build job runs on the same target
- **THEN** the cargo registry, git dependencies, and `target/` are restored from cache so the build skips already-compiled crates

### Requirement: Each release build SHALL be packaged into a native installer
After a successful per-platform build, the workflow SHALL run `cargo-packager` (driven by `Packager.toml`) to wrap the release binary into the platform-appropriate distributable format(s): a Windows NSIS `.exe` installer (current-user install mode) plus a portable `.zip` archive, a macOS `.app` bundle plus `.dmg` disk image, and a Linux `.deb` package, `.rpm` package, plus `.AppImage`. The packager config SHALL specify the product name (`Markion`), bundle identifier (`dev.markion.app`), version, category, and generated platform icon files (`assets/markion.ico`, `assets/markion.icns`, and `assets/markion.png`). For macOS x86_64 the packaging invocation SHALL pass `--target x86_64-apple-darwin` and `--binaries-dir target/x86_64-apple-darwin/release` (with `CARGO_BUILD_TARGET` set so the packager's before-packaging command rebuilds the same cross target), and the workflow SHALL verify after packaging that the produced macOS `.app` passes `codesign --verify --deep --strict`. The Windows portable archive SHALL contain the application binary and the same bundled resource payload as the NSIS install layout under a single top-level folder so extraction does not scatter files.

#### Scenario: Windows job produces an NSIS installer
- **WHEN** the Windows build job packages its binary
- **THEN** it emits a single NSIS `.exe` setup file that installs for the current user (no admin elevation required), creates Start Menu / Desktop shortcuts, and registers an Add/Remove-Programs entry
- **AND** it emits `markion_<version>_x64-portable.zip` containing a top-level portable folder with the application executable and the same resources the installer would place beside it

#### Scenario: macOS jobs produce app bundles and disk images for both architectures
- **WHEN** a macOS build job packages its binary
- **THEN** the arm64 job emits a `.app` bundle and `Markion_<version>_aarch64.dmg`, and the cross-compiled x86_64 job emits a `.app` bundle and `Markion_<version>_x64.dmg`; both use the `assets/markion.icns` app icon

#### Scenario: macOS packaging fails when the bundle is not sealed
- **WHEN** a macOS packaging run produces a `.app` that fails `codesign --verify --deep --strict` or does not carry an ad-hoc signature
- **THEN** the CI job fails and no artifact is published from that run

#### Scenario: Linux job produces a deb and an AppImage
- **WHEN** the Linux build job packages its binary
- **THEN** it emits a `.deb` package (amd64), an `.rpm` package (x86_64), and a portable `.AppImage`, all using the generated `assets/markion.png` icon and the `dev.markion.app` desktop entry identifier where the format supports a desktop entry

### Requirement: macOS installers SHALL be ad-hoc signed; Windows SHALL remain unsigned with documented bypasses
macOS `.app` bundles and DMGs SHALL be sealed with an ad-hoc code signature (`[macos] signing-identity = "-"` in the packager config; no Apple Developer Program membership, Developer ID certificate, or notarization). An ad-hoc seal keeps the signature *valid but untrusted*, for which Gatekeeper offers the System Settings → Privacy & Security → **Open Anyway** path, instead of the unsealed-bundle state that macOS reports as a hard "app is damaged" block with no override. Windows installers SHALL remain unsigned (no Authenticode certificate). The project SHALL document the resulting first-launch behavior on both platforms and SHALL document at least one no-Terminal bypass (Open Anyway), the quarantine-clearing Terminal fallback (`xattr -cr`), and the quarantine-free `curl -LO` download alternative. macOS releases SHALL ship native arm64 and x86_64 builds; no universal binary is provided, and Intel Macs SHALL NOT be told to use Rosetta (Rosetta 2 translates x86_64 to arm64 on Apple Silicon only and cannot run arm64 builds on Intel Macs).

#### Scenario: Ad-hoc signed macOS build warns but is overridable
- **WHEN** a user opens the distributed `.app` on macOS for the first time after a browser download
- **THEN** Gatekeeper reports that Apple cannot check the app for malicious software, and the user can launch it via System Settings → Privacy & Security → **Open Anyway** without using Terminal — documented behavior, not a defect
- **AND** the documented Terminal fallbacks (quarantine-clearing `xattr -cr`, or downloading with `curl -LO`, which does not set the quarantine attribute) remain available

#### Scenario: Intel Macs get a native build
- **WHEN** a user installs Markion on an Intel Mac
- **THEN** the release provides `Markion_<version>_x64.dmg` running natively on x86_64 macOS, and the documentation contains no Rosetta-based instruction for Intel Macs

#### Scenario: Unsigned Windows build warns the user
- **WHEN** a user runs the distributed NSIS installer on Windows for the first time
- **THEN** SmartScreen shows a "Windows protected your PC" warning, and the user must choose "More info → Run anyway" — this is documented behavior, not a defect

### Requirement: Published releases SHALL contain curated release information
The final GitHub Release description SHALL expand or replace auto-generated notes with a structured summary derived from the commits, diff, and completed OpenSpec changes since the previous tag. Unless the requester specifies another language arrangement, the summary SHALL be bilingual: the English summary first, followed by the corresponding Simplified Chinese version, and SHALL cover user-visible highlights and fixes, compatibility or migration information, available platform downloads, verification results, and a full comparison link. The final Release SHALL be a non-draft stable release unless a prerelease was explicitly requested.

#### Scenario: Generated notes omit direct commits
- **WHEN** GitHub's generated notes mention only merged pull requests or otherwise omit user-visible work
- **THEN** the operator supplements or replaces them with the complete curated summary before reporting the release complete

#### Scenario: Release has no migrations
- **WHEN** a version changes no persisted Markdown, preferences, or workspace data formats
- **THEN** the compatibility section explicitly states that no migration is required
- **AND** retains the documented unsigned/ad-hoc-installer warning when applicable

#### Scenario: Final release information is verified
- **WHEN** the tag workflow succeeds
- **THEN** the operator confirms that the Release is neither a draft nor an unintended prerelease
- **AND** confirms that the Windows NSIS installer, Windows portable `.zip`, macOS Apple Silicon DMG, macOS Intel x64 DMG, Linux amd64 DEB, Linux x86_64 RPM, and Linux x86_64 AppImage are attached
- **AND** confirms that the curated notes and comparison link are present

### Requirement: Tagged releases SHALL be mirrored to Aliyun OSS
Upon successful completion of the per-platform `build` jobs for a `v*` tag, the release workflow SHALL run a `mirror-oss` job that downloads the per-platform packaging artifacts, computes a SHA-256 digest for each installer package, generates a `manifest.json` describing the release, and uploads the Windows NSIS installer, Windows portable `.zip`, both macOS DMGs (arm64 and x64), Linux DEB, Linux RPM, Linux AppImage, `packager.toml`, `manifest.json`, and `sha256sums.txt` to a stable `${OSS_PREFIX}/latest/` path on the configured Aliyun OSS Bucket. The OSS endpoint, Bucket name, AccessKey ID, and AccessKey Secret SHALL come from repository secrets and SHALL NOT appear in the repository, the workflow file, or any OpenSpec artifact. The `mirror-oss` job SHALL depend on the `build` jobs and SHALL NOT depend on the `Publish GitHub Release` job; a failure of either job SHALL NOT prevent the other from running. A failure of the `mirror-oss` job SHALL be treated as an incomplete release requiring correction, even if the GitHub Release has already been published. The mirror SHALL overwrite any previous `${OSS_PREFIX}/latest/` objects so that the URL is a stable pointer to the newest release; per-tag history is retained only on GitHub Releases, not on OSS. The mirrored installers SHALL be byte-for-byte copies of the GitHub Release assets and SHALL NOT be code-signed or otherwise modified by the mirror step.

#### Scenario: Tagged release mirrors installers, config, and manifest to OSS
- **WHEN** a `v*` tag is pushed and all four native `build` jobs succeed
- **THEN** the `mirror-oss` job runs, downloads the per-platform packaging artifacts, and uploads the Windows NSIS installer, Windows portable `.zip`, both macOS DMGs, Linux DEB, Linux RPM, Linux AppImage, `packager.toml`, `manifest.json`, and `sha256sums.txt` to `${OSS_PREFIX}/latest/` on the configured Bucket
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
- **THEN** the manifest contains the release version (without the leading `v`), the tag name, an ISO-8601 publication timestamp, and a map from platform identifier (`windows-x86_64`, `windows-x86_64-portable`, `macos-aarch64`, `macos-x86_64`, `linux-amd64`, `linux-rpm`, `linux-appimage`) to the installer package filename
- **AND** the manifest remains available as mirror metadata without being required for the initial in-app update-check verification
