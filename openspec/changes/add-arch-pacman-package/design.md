## Context

See `proposal.md` — Why. The Ubuntu 22.04 release job already builds the Linux x86_64 binary, creates DEB and AppImage packages with cargo-packager, converts the DEB payload to RPM, verifies every package's resources, and uploads one `markion-linux-x86_64` artifact. GitHub publication and OSS mirroring consume that artifact automatically. The active `prefer-appimage-update-on-arch` change currently maps Arch-family browser downloads to AppImage; this change supersedes that mapping only for non-AppImage launches.

The Windows host used for local development has no Docker, Arch userspace, `makepkg`, `bsdtar` or `zstd`. Native package construction and pacman installation therefore need GitHub's Linux runners for end-to-end verification. No rendering or editing state is involved, so all per-version Markdown caches and UI invariants remain unchanged.

## Goals / Non-Goals

**Goals:**

- Produce a conventional pacman package without rebuilding or modifying the application binary and resources.
- Use real Arch tooling for package construction and installation validation, while retaining the existing cross-platform resource verifier.
- Add exactly one artifact to existing GitHub/OSS publication paths and preserve mirror-repair support.
- Select the new asset only where it matches the running Linux installation context.

**Non-Goals:**

- Building on an Arch host instead of the established Ubuntu 22.04 compatibility baseline.
- Supporting AArch64 or other new targets, publishing to AUR, hosting a pacman repository, or adding package-manager signatures.
- Adding pacman to `update.json` or implementing privileged in-app package installation.
- Claiming Omarchy desktop acceptance from an Arch container installation check.

## Decisions

### Build with `makepkg` in a disposable Arch Linux container

Add `scripts/build-linux-arch.sh` to extract `dist/markion_<version>_amd64.deb`, generate a temporary `PKGBUILD`, and run `makepkg` with `PKGEXT=.pkg.tar.zst`. The GitHub Ubuntu job invokes the script through `docker run` with `archlinux:base-devel`; `makepkg` runs as an unprivileged builder, while the surrounding container step performs package installation and copies the finished artifact to `dist/`.

Use the conventional output name `markion-<version>-1-x86_64.pkg.tar.zst`. The package release is independently overridable for packaging-only respins, defaulting to `1`. Explicit Arch dependencies map the existing Linux runtime requirements to `wayland`, `libxkbcommon`, `libxkbcommon-x11`, `libxcb`, `vulkan-icd-loader`, `fontconfig`, `alsa-lib` and `glib2`; a software Vulkan driver is an optional dependency rather than a bundled second copy of lavapipe.

**Alternatives considered:** hand-writing `.PKGINFO` and a zstd tar stream on Ubuntu would avoid Docker but duplicates makepkg's metadata and MTREE behavior and would not prove pacman compatibility. A native Arch compilation would raise the glibc baseline and repeat an expensive build without improving this package's payload identity.

### Reuse the DEB file tree as the package payload

The `PKGBUILD` `package()` step copies the extracted DEB root unchanged. This preserves the executable path, desktop entry, icon, examples, licenses and MarkNice resource layout already consumed by Markion. The build script rejects a missing DEB, unexpected version or wrong machine architecture instead of silently producing a partial package.

The same Arch container performs `pacman -U` with dependency resolution, inspects the installed package through `pacman -Q`, and checks the installed executable and resource roots. This is an installation/CLI gate, not GUI acceptance.

**Alternative considered:** extracting the AppImage would include its bundled software Vulkan stack and AppImage-specific launcher, changing the native-package layout and unnecessarily inflating it.

### Keep pacman in the existing Linux artifact and release dependency graph

Run the Arch container as an additional step in the existing Ubuntu build job, after DEB/RPM creation and before artifact upload. Extend `verify_formats` with `pacman`; teach `scripts/verify-packaged-workspace.ps1` to extract `*.pkg.tar.zst` with `bsdtar` and run the existing examples/workspace checks. Install the small host extraction tools in the existing Linux dependency step.

Because the pacman file is inside `markion-linux-x86_64`, the existing release download already attaches it. The `build` job remains the packaging gate; no new artifact name or downstream job dependency is needed. Explicitly add the package to mirror repair, `sha256sums.txt`, the `linux-arch-x86_64` manifest entry, OSS upload and HTTP verification lists.

### Make browser download selection context-sensitive

Keep the AppImage branch first in `browser_download_url_with_os_release`, so an AppImage launch always retains its signed update asset. For non-AppImage Linux x86_64, rename the existing distribution predicate to reflect Arch-family detection and select the suffix `-x86_64.pkg.tar.zst`; retain `_amd64.deb` for other or unknown distributions. Match the exact release asset URL and return the existing recoverable error when it is absent.

Extend hermetic tests with pacman assets and Arch, Omarchy, Manjaro, Debian, missing os-release, AppImage-on-Arch and missing-package cases. Do not change signed manifest endpoints, updater formats, installation privileges, localization or any document state.

### Reconcile the active Arch update proposal without inventing native evidence

Before syncing or archiving this change, rebase its update requirement on the then-current stable spec and reconcile `prefer-appimage-update-on-arch` so both changes cannot restore conflicting Arch-family mappings. Its pending Omarchy manual task remains pending unless actually executed; Arch container installation cannot complete it.

## Risks / Trade-offs

- **[Rolling Arch container or keyring changes break CI]** → Use the official `archlinux:base-devel` image, refresh package databases once, fail on package/install errors, and keep the resulting package as a workflow artifact for diagnosis. Pinning an old image would increase supply-chain and compatibility risk.
- **[A dependency name is valid on Debian/Fedora but not Arch]** → `pacman -U` with dependency resolution is mandatory; package inspection alone is insufficient.
- **[The DEB payload quietly diverges from other packages]** → Build every Linux format from the same job and run the existing complete resource verifier on the extracted pacman payload.
- **[Large assets lengthen OSS upload and repair]** → Add the pacman file to the same multipart upload, digest and HTTP checks rather than creating a separate channel.
- **[Users mistake a native package for a signed self-update]** → Keep it out of `update.json`, document `sudo pacman -U <package>`, and describe Minisign coverage only for NSIS/AppImage.
- **[Arch installation is overreported as Omarchy validation]** → Label CI evidence precisely and retain Omarchy desktop interaction as a separate native acceptance row.

## Migration Plan

No document, preference, workspace or updater-metadata migration is required. New CI steps run on branch/PR builds before any tag, allowing package construction and installation to be rehearsed without publication. Release automation gains one required asset and mirror object; existing assets and signatures remain byte-for-byte governed by their current steps.

If the new package fails, do not publish a partial release silently: fix the package or explicitly remove the unverified scope through a revised change before tagging. Rollback of an already published public tag remains prohibited without explicit maintainer authorization; mirror repair uses the existing manual workflow and must include the pacman asset.
