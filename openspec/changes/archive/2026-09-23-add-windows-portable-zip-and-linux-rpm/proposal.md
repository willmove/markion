## Why

Markion releases currently ship only an NSIS installer on Windows and `.deb`/`.AppImage` on Linux. Users who want a no-install Windows copy, and Fedora/RHEL-family users who prefer native RPM packages, have no matching download. Expanding the installer matrix closes those distribution gaps before the next tagged release.

## What Changes

- Add a Windows x86_64 portable archive (`.zip`) that contains the release binary and bundled resources without requiring an installer.
- Add a Linux x86_64 `.rpm` package alongside the existing `.deb` and `.AppImage`.
- Verify both new packages contain the same pinned MarkNice publishing workspace as every other native package.
- Attach both new packages to the GitHub Release and mirror them to the Aliyun OSS `latest/` prefix with the existing installer set.
- Document the new downloads in the release runbook and curated release-note template.

Non-goals: changing the in-app updater asset mapping, adding Authenticode/code signing, producing additional CPU architectures, or changing how the portable archive stores user preferences (it keeps the normal per-user config location).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `release-packaging`: Expand the required per-platform installer set with a Windows portable `.zip` and a Linux `.rpm`, require those packages to contain the MarkNice workspace, and include them in GitHub Release assets and OSS mirroring.

## Impact

- `.github/workflows/release.yml`: Windows/Linux format lists, portable zip packaging step, package verification, OSS mirror upload list, and `manifest.json`/`sha256sums` coverage.
- `packager.toml`: default `formats` and a top-level `[rpm]` dependency table.
- `scripts/verify-packaged-workspace.ps1`: accept and extract `zip` and `rpm` packages.
- `docs/release-process.md`: download inventory, mirror object list, and final verification checklist.
- No application source, cached Markdown state, or workspace-member dependency changes.
