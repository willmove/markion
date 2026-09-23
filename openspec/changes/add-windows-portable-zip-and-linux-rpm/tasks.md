## 1. Packager and package scripts

- [x] 1.1 Document in `packager.toml` that cargo-packager covers nsis/app/dmg/deb/appimage only and that CI additionally emits a Windows portable `.zip` and a Linux `.rpm`; leave default `formats` unchanged — verify the config still has no unknown tables
- [x] 1.2 Add `scripts/build-windows-portable.ps1` to stage `markion.exe` + `assets/` + `THIRD_PARTY_NOTICES.md` under `Markion/` and write `dist/markion_<version>_x64-portable.zip` — verify the zip extracts with resources next to the executable
- [x] 1.3 Add `scripts/build-linux-rpm.sh` + a small `.spec` that installs the extracted DEB payload via `rpmbuild` and writes `dist/markion_<version>_x86_64.rpm` — verify the produced RPM lists the same payload paths as the DEB

## 2. CI packaging and verification

- [x] 2.1 After Linux `cargo packager --formats deb,appimage`, run `scripts/build-linux-rpm.sh` and verify the RPM with the extended workspace checker
- [x] 2.2 After Windows NSIS packaging, run `scripts/build-windows-portable.ps1` and keep the existing NSIS path unchanged — verify both packages appear in the uploaded artifact listing
- [x] 2.3 Teach `scripts/verify-packaged-workspace.ps1` to accept `zip` and `rpm` formats, extract them, and run the same MarkNice workspace + export-bundle closure checks — verify a missing workspace file fails the script for both formats
- [x] 2.4 Update `mirror-oss` download patterns, `sha256sums`, `manifest.json` platform map, upload list, and public-reachability checks to include the portable zip and RPM — verify the job list comments and scripts mention the full six-package matrix

## 3. Documentation

- [x] 3.1 Update `docs/release-process.md` asset checklist, OSS object list, release-note Downloads template, and final verification bullets for the Windows portable zip and Linux RPM — verify the runbook names both new packages in every inventory that previously listed only four installers

## 4. Integration verification

- [x] 4.1 Run `openspec validate add-windows-portable-zip-and-linux-rpm` and `cargo test --workspace` — verify validation passes and tests are green without touching cached Markdown invariants
