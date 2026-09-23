## Context

See `proposal.md` for motivation. Today `cargo-packager` 0.11.8 produces NSIS, `.app`/`.dmg`, `.deb`, and `.AppImage` from `packager.toml`, and `release.yml` publishes those four installers to GitHub Releases and Aliyun OSS. `scripts/verify-packaged-workspace.ps1` extracts each package format and asserts the pinned MarkNice workspace is present and closed. The in-app updater continues to map Windows to the signed NSIS installer and Linux to `.deb`/AppImage; that mapping is out of scope here.

## Goals / Non-Goals

**Goals:**

- Ship a Windows portable `.zip` and a Linux `.rpm` as first-class release assets.
- Keep every native package format (including the new two) self-contained with the MarkNice workspace and verified before upload.
- Mirror the new packages with the existing set so GitHub and OSS remain complete and consistent.

**Non-Goals:**

- Changing `src/app/update.rs` download mapping or updater signing.
- Introducing a new packager tool or replacing `cargo-packager`.
- Producing Windows ARM, macOS x86_64, or Linux aarch64 packages.
- Storing portable-mode user data beside the executable (config stays in the normal per-user location).

## Decisions

1. **RPM assembled from the DEB payload with `rpmbuild`** — cargo-packager 0.11.8's `PackageFormat` has no RPM variant (supported: app/dmg/wix/nsis/deb/appimage/pacman), so `packager.toml` cannot emit `.rpm`. The Linux job packages `deb,appimage` as today, extracts the DEB file tree, and runs a checked-in `rpmbuild` spec that installs that same tree, preserving the proven resource layout Markion's `bundled_resource_path` / `discover_workspace_assets` already resolve. Fedora/RHEL-style runtime dependencies are declared on the spec to match the existing `[deb]` set. Alternatives rejected: adding a non-existent `[rpm]` packager table (config is `deny_unknown_fields`); converting with `alien`/`fpm` (extra Ruby/alien toolchain and less transparent file lists).

2. **Portable zip assembled in the Windows job after NSIS packaging** — cargo-packager 0.11.x has no `zip` format either. The Windows job therefore stages `markion.exe` plus the same `resources` payload (`assets/`, `THIRD_PARTY_NOTICES.md`) that NSIS would install, under a single top-level `Markion/` folder, and compresses it to `markion_<version>_x64-portable.zip`. Alternative rejected: zipping the entire `target/release` tree would ship build junk and omit a clean portable layout.

3. **Stable portable zip naming** — Use `markion_<version>_x64-portable.zip` so it sorts beside `markion_<version>_x64-setup.exe` and is greppable in release tooling and the mirror upload list.

4. **Package verification extends the existing MarkNice closure check** — `verify-packaged-workspace.ps1` gains `zip` (Expand-Archive) and `rpm` (`rpm2cpio | cpio` on Linux runners) extractors, reusing the same `bundle-manifest.json` + `verify-bundle` path as NSIS/DEB/AppImage. This keeps "incomplete bundle blocks publication" true for the new formats.

5. **Mirror and release notes treat the new files as required assets** — `mirror-oss` upload/verify lists, `manifest.json` platform map, `sha256sums.txt`, and `docs/release-process.md` final checks include the portable zip and RPM so a "complete release" means the full matrix, not the previous four installers.

No application render/edit/cache invariants are touched; this change is packaging and CI only.

## Risks / Trade-offs

- [cargo-packager RPM output name may differ slightly from Debian-style naming] → Discover the produced filename from `dist/` after packaging and use that name in mirror lists; pin expected suffixes (`*.rpm`, `*_x64-portable.zip`) in verification rather than hard-coding a full filename until confirmed by the first CI run.
- [RPM dependency names differ across Fedora/RHEL/openSUSE] → Declare the Fedora/RHEL names that match the existing Debian runtime set; document that other RPM distros may need equivalent packages. AppImage remains the most portable Linux option.
- [Portable zip is unsigned like the NSIS installer] → Document SmartScreen expectations for extracted `markion.exe`; no Authenticode scope creep.
- [Windows zip step must include MarkNice assets exactly like NSIS] → Stage from the same `resources` list and run the same workspace verifier on the extracted tree before upload.

## Migration Plan

No data migration. Existing installs, preferences, and documents are unaffected. The next tagged release simply publishes two additional downloadable packages. Rollback is omitting the new formats from the matrix (no client compatibility break).

## Open Questions

None that change scope or task breakdown. Exact cargo-packager RPM filename will be confirmed in the first packaging CI run and locked into the mirror list if it differs from `markion-<version>-1.x86_64.rpm`.
