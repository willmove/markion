## RENAMED Requirements

### FROM: macOS installers SHALL be ad-hoc signed; Windows SHALL remain unsigned with documented bypasses
### TO: macOS installers SHALL be notarized when Developer ID credentials are configured; Windows SHALL remain unsigned with documented bypasses

## MODIFIED Requirements

### Requirement: Each release build SHALL be packaged into a native installer
After a successful per-platform build, the workflow SHALL run `cargo-packager` (driven by `Packager.toml`) to wrap the release binary into the platform-appropriate distributable format(s): a Windows NSIS `.exe` installer (current-user install mode) plus a portable `.zip` archive, a macOS `.app` bundle plus `.dmg` disk image, and a Linux `.deb` package, `.rpm` package, plus `.AppImage`. The packager config SHALL specify the product name (`Markion`), bundle identifier (`dev.markion.app`), version, category, and generated platform icon files (`assets/markion.ico`, `assets/markion.icns`, and `assets/markion.png`). For macOS x86_64 the packaging invocation SHALL pass `--target x86_64-apple-darwin` and `--binaries-dir target/x86_64-apple-darwin/release` (with `CARGO_BUILD_TARGET` set so the packager's before-packaging command rebuilds the same cross target), and the workflow SHALL verify after packaging that the produced macOS `.app` passes `codesign --verify --deep --strict` and carries the signature kind selected for that run (ad-hoc, Developer ID, or Developer ID with stapled notarization tickets on both the `.app` and the `.dmg`). The Windows portable archive SHALL contain the application binary and the same bundled resource payload as the NSIS install layout under a single top-level folder so extraction does not scatter files.

#### Scenario: Windows job produces an NSIS installer
- **WHEN** the Windows build job packages its binary
- **THEN** it emits a single NSIS `.exe` setup file that installs for the current user (no admin elevation required), creates Start Menu / Desktop shortcuts, and registers an Add/Remove-Programs entry
- **AND** it emits `markion_<version>_x64-portable.zip` containing a top-level portable folder with the application executable and the same resources the installer would place beside it

#### Scenario: macOS jobs produce app bundles and disk images for both architectures
- **WHEN** a macOS build job packages its binary
- **THEN** the arm64 job emits a `.app` bundle and `Markion_<version>_aarch64.dmg`, and the cross-compiled x86_64 job emits a `.app` bundle and `Markion_<version>_x64.dmg`; both use the `assets/markion.icns` app icon

#### Scenario: macOS packaging fails when the bundle does not carry the selected signature
- **WHEN** a macOS packaging run produces a `.app` that fails `codesign --verify --deep --strict`, or that lacks the signature its run selected — `Signature=adhoc` in ad-hoc mode; a `Developer ID Application` authority, a team identifier, and the hardened runtime in Developer ID mode; and, when notarization was selected, valid stapled tickets on the `.app` and `.dmg` with `spctl` reporting `source=Notarized Developer ID`
- **THEN** the CI job fails and no artifact is published from that run

#### Scenario: Linux job produces a deb and an AppImage
- **WHEN** the Linux build job packages its binary
- **THEN** it emits a `.deb` package (amd64), an `.rpm` package (x86_64), and a portable `.AppImage`, all using the generated `assets/markion.png` icon and the `dev.markion.app` desktop entry identifier where the format supports a desktop entry

### Requirement: macOS installers SHALL be notarized when Developer ID credentials are configured; Windows SHALL remain unsigned with documented bypasses
The committed packager config SHALL keep the ad-hoc identity (`[macos] signing-identity = "-"`) so builds without Apple credentials (forks, pull requests from forks, local runs) still produce a sealed, valid-but-untrusted bundle. When the repository secrets `APPLE_SIGNING_IDENTITY` (a `Developer ID Application:` identity), `APPLE_CERTIFICATE` (base64 `.p12`), and `APPLE_CERTIFICATE_PASSWORD` are all configured, both macOS jobs SHALL sign the `.app` (hardened runtime, secure timestamp) and the `.dmg` with that Developer ID identity. When App Store Connect API key secrets `APPLE_API_KEY`, `APPLE_API_ISSUER`, and `APPLE_API_PRIVATE_KEY` are also configured, `v*` tag builds and manual `workflow_dispatch` builds SHALL notarize and staple both the `.app` and the `.dmg`, and SHALL fail rather than publish when Apple does not accept a submission. A partially configured credential set SHALL fail the job, and a `v*` tag build that does not notarize SHALL emit a workflow warning. No Apple credential SHALL appear in the repository, the workflow file, an OpenSpec artifact, or job logs. Windows installers SHALL remain unsigned (no Authenticode certificate). The project SHALL document the first-launch behavior of notarized and non-notarized macOS builds and of Windows builds, SHALL NOT claim that non-notarized macOS builds can always be approved through **Open Anyway**, and SHALL document the quarantine-clearing Terminal fallback (`xattr -cr`) and the quarantine-free `curl -LO` download for non-notarized builds, plus a maintainer guide for provisioning the Apple credentials. macOS releases SHALL ship native arm64 and x86_64 builds; no universal binary is provided, and Intel Macs SHALL NOT be told to use Rosetta (Rosetta 2 translates x86_64 to arm64 on Apple Silicon only and cannot run arm64 builds on Intel Macs).

#### Scenario: Notarized macOS build opens after a browser download
- **WHEN** a user downloads a notarized release DMG in a browser, drags Markion to Applications, and opens it for the first time
- **THEN** Gatekeeper accepts the Developer ID signature and stapled notarization ticket and shows at most the standard "downloaded from the Internet" confirmation, and Markion launches without any Terminal command or System Settings change

#### Scenario: Builds without Apple credentials stay ad-hoc sealed
- **WHEN** a macOS job runs without the Developer ID secrets (for example a pull request from a fork)
- **THEN** the produced `.app` is ad-hoc sealed exactly as before and the job succeeds
- **AND** a `v*` tag build in this state emits a workflow warning that its macOS DMGs are not notarized

#### Scenario: Non-notarized macOS build documents a reliable fallback
- **WHEN** a user opens a non-notarized (ad-hoc) build downloaded in a browser and macOS refuses it, including with the "is damaged and can't be opened" wording
- **THEN** the documentation explains this is Gatekeeper policy for non-notarized apps rather than a corrupt download, and gives the `xattr -cr` command and the `curl -LO` download as the working alternatives

#### Scenario: Partial Apple credentials fail the build
- **WHEN** only some of the Developer ID secrets, or notarization secrets without the Developer ID secrets, are configured
- **THEN** the macOS job fails with an error naming the missing secret instead of silently publishing an ad-hoc build

#### Scenario: Notarization rejection blocks publication
- **WHEN** Apple's notary service returns a status other than `Accepted` for the `.app` or the `.dmg` during a notarizing build
- **THEN** the job prints the notary log, fails, and no macOS artifact from that run is published

#### Scenario: Intel Macs get a native build
- **WHEN** a user installs Markion on an Intel Mac
- **THEN** the release provides `Markion_<version>_x64.dmg` running natively on x86_64 macOS, and the documentation contains no Rosetta-based instruction for Intel Macs

#### Scenario: Unsigned Windows build warns the user
- **WHEN** a user runs the distributed NSIS installer on Windows for the first time
- **THEN** SmartScreen shows a "Windows protected your PC" warning, and the user must choose "More info → Run anyway" — this is documented behavior, not a defect
