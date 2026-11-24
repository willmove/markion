## Why

Markion releases provide DEB, RPM and AppImage packages for Linux, but no native package for pacman-based distributions. Arch Linux and Omarchy users therefore cannot install and manage Markion through their normal system package manager, and the update dialog cannot offer them a matching native download.

## What Changes

- Add a Linux x86_64 pacman package named `markion-<version>-<release>-x86_64.pkg.tar.zst`, assembled with Arch Linux `makepkg` from the same application and resource payload used by the existing Linux packages.
- Declare Arch-native runtime dependencies and verify package metadata, installed layout, bundled examples and the pinned MarkNice workspace. Add an Arch userspace installation check to the release gates.
- Attach the pacman package to GitHub Releases, include it in SHA-256 checksums and OSS `manifest.json`, mirror it to Aliyun OSS, and include it in mirror-repair and final-release verification.
- On non-AppImage Arch-family Linux installations, including Omarchy, offer the pacman package for browser-download updates. AppImage launches retain the signed AppImage update path; other distributions retain their existing mapping.
- Update release documentation, download instructions and native acceptance coverage for the additional package. This change adds distribution packaging only; it does not alter document-version caches, syntax-highlighting memoization, cached text handles or file-tree row bounds.

**Non-goals:** AUR submission, a pacman repository or signing key, automatic `pacman -U` installation from the app, new CPU architectures, and changes to existing Windows/macOS/Linux package formats.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `release-packaging`: add the Arch/Omarchy pacman artifact to native packaging, bundled-resource verification, GitHub/OSS publication, release acceptance and Linux distribution-family update downloads.

## Impact

- **CI and packaging:** `.github/workflows/release.yml`, a new Arch package build script, `scripts/verify-packaged-workspace.ps1`, and packaging documentation.
- **Update selection:** `src/app/update.rs` and its hermetic os-release/asset-mapping tests; no new updater endpoint, signing key or self-install mechanism.
- **Release metadata:** GitHub assets, OSS mirror and repair inputs, `manifest.json`, and `sha256sums.txt` gain one package; `update.json` remains limited to the existing signed Windows NSIS and Linux AppImage entries.
- **Documentation:** release runbook/template, download guidance and platform acceptance matrix.
- **Dependencies:** CI uses an Arch Linux userspace with `makepkg`/`pacman`; no Rust crate or application runtime dependency is added.
