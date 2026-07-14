## MODIFIED Requirements

### Requirement: Each release build SHALL be packaged into a native installer
After a successful per-platform build, the workflow SHALL run `cargo-packager` (driven by `Packager.toml`) to wrap the release binary into the platform-appropriate distributable format(s): a Windows NSIS `.exe` installer (current-user install mode), a macOS `.app` bundle plus `.dmg` disk image, and a Linux `.deb` package plus `.AppImage`. The packager config SHALL specify the product name (`Markion`), bundle identifier (`dev.markion.app`), version, category, and the per-platform icon files.

#### Scenario: Windows job produces an NSIS installer
- **WHEN** the Windows build job packages its binary
- **THEN** it emits a single NSIS `.exe` setup file that installs for the current user (no admin elevation required), creates Start Menu / Desktop shortcuts, and registers an Add/Remove-Programs entry

#### Scenario: macOS job produces an app bundle and disk image
- **WHEN** the macOS build job packages its binary
- **THEN** it emits a `.app` bundle and a `.dmg` disk image, both arm64 (Apple Silicon); the app icon is `markion.icns`

#### Scenario: Linux job produces a deb and an AppImage
- **WHEN** the Linux build job packages its binary
- **THEN** it emits a `.deb` package (amd64) and a portable `.AppImage`, both using the `markion.png` icon and the `dev.markion.app` desktop entry identifier
