## MODIFIED Requirements

### Requirement: Each release build SHALL be packaged into a native installer
After a successful per-platform build, the workflow SHALL run `cargo-packager` (driven by `Packager.toml`) to wrap the release binary into the platform-appropriate distributable format(s): a Windows NSIS `.exe` installer (current-user install mode) plus a portable `.zip` archive, a macOS `.app` bundle plus `.dmg` disk image, and a Linux `.deb` package, `.rpm` package, plus `.AppImage`. The packager config SHALL specify the product name (`Markion`), bundle identifier (`dev.markion.app`), version, category, and generated platform icon files (`assets/markion.ico`, `assets/markion.icns`, and `assets/markion.png`). For macOS x86_64 the workflow SHALL stage the cross-built binary in the packager's configured binaries directory and SHALL invoke the packager with `--target x86_64-apple-darwin` and `CARGO_BUILD_TARGET=x86_64-apple-darwin`. The workflow SHALL verify the executable architecture, bundle signature and notarization ticket inside each public DMG, and the signature and ticket of the DMG itself. The Windows portable archive SHALL contain the application binary and the same bundled resource payload as the NSIS install layout under a single top-level folder so extraction does not scatter files.

#### Scenario: Windows job produces an NSIS installer
- **WHEN** the Windows build job packages its binary
- **THEN** it emits a single NSIS `.exe` setup file that installs for the current user (no admin elevation required), creates Start Menu / Desktop shortcuts, and registers an Add/Remove-Programs entry
- **AND** it emits `markion_<version>_x64-portable.zip` containing a top-level portable folder with the application executable and the same resources the installer would place beside it

#### Scenario: macOS jobs produce app bundles and disk images for both architectures
- **WHEN** a macOS build job packages its binary
- **THEN** the arm64 job emits a `.app` bundle and `Markion_<version>_aarch64.dmg`, and the cross-compiled x86_64 job emits a `.app` bundle and `Markion_<version>_x64.dmg`; both use the `assets/markion.icns` app icon

#### Scenario: macOS packaging fails when the bundle is not sealed
- **WHEN** an ad-hoc CI bundle fails `codesign --verify --deep --strict` or lacks its ad-hoc signature, or a release DMG or its embedded app fails the Developer ID or notarization checks
- **OR** the embedded executable's architecture does not match the release matrix target
- **THEN** the CI job fails and no artifact is published from that run

#### Scenario: Linux job produces a deb and an AppImage
- **WHEN** the Linux build job packages its binary
- **THEN** it emits a `.deb` package (amd64), an `.rpm` package (x86_64), and a portable `.AppImage`, all using the generated `assets/markion.png` icon and the `dev.markion.app` desktop entry identifier where the format supports a desktop entry

#### Scenario: macOS job produces an app bundle and disk image
- **WHEN** the Apple Silicon job packages its binary
- **THEN** it emits an arm64 `.app` bundle and `.dmg` disk image using the `assets/markion.icns` app icon

## ADDED Requirements

### Requirement: Public macOS installers SHALL carry Developer ID signatures and notarization tickets
For tagged releases and manual signing rehearsals, both macOS jobs SHALL sign with Developer ID Application and verify the signatures and stapled Apple tickets on the final packages before upload. App signatures SHALL use the hardened runtime and a secure timestamp. Apple credentials SHALL come from GitHub Actions Secrets and stay out of source, artifacts and logs. Documentation SHALL describe first launch and identify older ad-hoc releases.

#### Scenario: Notarized macOS build opens after a browser download
- **WHEN** a user downloads a notarized release DMG in a browser, drags Markion to Applications, and opens it for the first time
- **THEN** Gatekeeper accepts the Developer ID signature and stapled notarization ticket and shows at most the standard "downloaded from the Internet" confirmation, and Markion launches without any Terminal command or System Settings change

#### Scenario: Development builds use ad-hoc signatures
- **WHEN** a local build, branch push or pull request builds a macOS package
- **THEN** the workflow uses the committed ad-hoc identity without reading Apple credentials

#### Scenario: Final disk image is verified before upload
- **WHEN** a signing job has stapled its DMG
- **THEN** the job verifies both the DMG and its mounted app's signatures and tickets
- **AND** Gatekeeper accepts both when assessments are enabled

#### Scenario: Missing or partial credentials block publication
- **WHEN** a tag build or manual signing rehearsal lacks any required Apple credential
- **THEN** the macOS job fails and identifies the missing secret names
- **AND** the release workflow publishes no packages from that run

#### Scenario: Notarization rejection blocks publication
- **WHEN** Apple's notary service returns a status other than `Accepted` for the `.app` or the `.dmg` during a notarizing build
- **THEN** the job prints the notary log, fails, and no macOS artifact from that run is published

#### Scenario: Intel Macs get a native build
- **WHEN** a user installs Markion on an Intel Mac
- **THEN** the release provides `Markion_<version>_x64.dmg` running natively on x86_64 macOS, and the documentation contains no Rosetta-based instruction for Intel Macs

### Requirement: Windows installers SHALL remain unsigned with documented first-launch guidance
Windows installers SHALL remain unsigned by Authenticode. The project SHALL document the resulting SmartScreen first-launch behavior.

#### Scenario: Unsigned Windows build warns the user
- **WHEN** a user runs the distributed NSIS installer on Windows for the first time
- **THEN** SmartScreen shows a "Windows protected your PC" warning, and the user must choose "More info → Run anyway" — this is documented behavior, not a defect

## REMOVED Requirements

### Requirement: macOS installers SHALL be ad-hoc signed; Windows SHALL remain unsigned with documented bypasses
**Reason**: Public macOS installers require Developer ID signing and notarization. The separate Windows requirement preserves the existing behavior.
**Migration**: Maintainers configure the Apple credentials for public tags. Existing historical assets retain the signature with which they were published.
