## 1. Build and verify the pacman package

- [x] 1.1 Add `scripts/build-linux-arch.sh` to extract the existing DEB payload, generate the Arch `PKGBUILD`, run unprivileged `makepkg`, emit `markion-<version>-<release>-x86_64.pkg.tar.zst`, and perform real `pacman -U`/`pacman -Q` installation checks; verify with `bash -n` and a branch-workflow Arch container run that confirms metadata, dependencies, executable and installed resources.
- [x] 1.2 Extend `scripts/verify-packaged-workspace.ps1` with a `pacman` format and add the required extraction tools and package step to `.github/workflows/release.yml`; verify a non-tag Linux workflow run builds the package and passes the existing complete examples and MarkNice workspace checks without changing document-version caches.

## 2. Select the native asset for Arch-family updates

- [x] 2.1 Update `src/app/update.rs` so AppImage launches keep AppImage, non-AppImage Arch/Omarchy/Manjaro launches select `-x86_64.pkg.tar.zst`, and other or unknown Linux distributions keep DEB; add hermetic asset-mapping tests for those branches and a missing pacman asset, then run the focused update tests with no failures.
- [x] 2.2 Reconcile the active `prefer-appimage-update-on-arch` planning artifacts so their AppImage mapping cannot overwrite this change at a later sync/archive; leave its Omarchy manual verification unchecked unless actually performed and validate the affected change artifacts.

## 3. Integrate publication and documentation

- [x] 3.1 Add the pacman artifact to OSS mirror repair, `sha256sums.txt`, `manifest.json` as `linux-arch-x86_64`, upload and public HTTP verification while leaving `update.json` restricted to signed NSIS/AppImage entries; verify the non-tag workflow assembles all expected metadata inputs and inspect the diff to ensure no signing-secret or updater-format expansion.
- [x] 3.2 Update `packager.toml` packaging comments, `docs/release-process.md`, the bilingual release-note template, platform acceptance cases/matrix, and maintained download/install guidance with the eight-package matrix and `sudo pacman -U` instructions; verify links, asset names, package counts and statements distinguish Arch installation evidence from unrun Omarchy desktop acceptance.
- [x] 3.3 Prepare concise bilingual `v0.6.3` release notes covering changes since `v0.6.2`, the new Arch/Omarchy-compatible package, actual signing status and unverified scope; verify every claim against the complete tag-to-tag diff and the release verification record.

## 4. Validate before release

- [x] 4.1 Run `openspec validate add-arch-pacman-package --strict`, shell/PowerShell syntax checks, `cargo test --workspace`, the bundle/examples/fixture checks and `git diff --check`; preserve outputs and resolve every failure introduced by this change.
- [x] 4.2 Push the implementation to `main`, monitor the non-tag `release.yml` run, and download the Linux artifact to confirm the pacman package exists, passes Arch installation/resource gates and does not displace DEB/RPM/AppImage; do not tag until the complete native candidate matrix succeeds.
- [x] 4.3 Execute applicable candidate P01/P02/P06/P07 acceptance for the new package on Arch and, where available, Omarchy, recording package/program hashes and actual observations; record missing native environments as BLOCKED/NOT_RUN and do not claim release readiness without the required results or an explicit release-specific maintainer exception.
