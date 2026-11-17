# Proposal: bundle-appimage-software-vulkan

## Why

The Linux AppImage dies within a second on machines without a Vulkan driver: GPUI's blade renderer finds no Vulkan device and the X11 client panics with `Unable to init GPU context: NoSupportedDeviceFound`. This is exactly what failed the AppImage catalog test for v0.4.8 (AppImage/appimage.github.io#6581, ubuntu-22.04 + Xvfb + firejail, Mesa GL but no `mesa-vulkan-drivers`), and it is what users in VMs, cloud desktops and minimal installs hit. Markion already renders correctly on Mesa's software Vulkan driver (lavapipe), so a portable AppImage should carry that driver instead of requiring users to install one, and must say clearly what is wrong when no renderer can start at all.

## What Changes

- **Bundled software Vulkan stack in the AppImage**: the Linux release pipeline builds Mesa lavapipe (statically linked against LLVM, so no `libLLVM` dependency) plus the Khronos Vulkan loader on the same ubuntu-22.04 baseline as the app, and places them under `usr/lib/markion-vulkan/` in the AppImage together with their license texts. The `.deb`/`.rpm` packages are unchanged and keep using the system Vulkan stack.
- **Automatic, hardware-first fallback**: Markion always tries the host Vulkan stack first. Only when renderer initialization fails and the bundled stack is present does Markion restart itself once with the Vulkan loader pointed at the bundled lavapipe ICD (implicit host layers disabled). Machines with a working GPU driver never load the bundled stack and keep hardware rendering. `MARKION_SOFTWARE_VULKAN=1` forces the bundled stack; `MARKION_SOFTWARE_VULKAN=0` disables the fallback.
- **Clear failure instead of a bare panic** (Linux slice of `harden-linux-gpu-init` Phase A): when no renderer can be initialized — including after the software fallback — Markion logs the error, prints a localized explanation with remediation steps to stderr, shows it in a native dialog when a dialog tool (`zenity`, `kdialog`, `xmessage`) is available, and exits non-zero.
- **Catalog-equivalent CI smoke test**: the release workflow launches the freshly built AppImage on an ubuntu-22.04 runner with Xvfb, a window manager and `firejail --appimage`, with no system Vulkan driver, and fails unless a visible Markion window appears.
- The AppImage permission normalization from PR #17 and the AppImage updater signature/`update.json` flow are kept as they are; the bundling step runs before permission normalization and before signing.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `release-packaging`: adds the requirement that the Linux AppImage starts and shows a window on hosts without a usable Vulkan driver by falling back to a bundled software Vulkan stack, that hardware Vulkan stays preferred, and that the release pipeline proves this with a catalog-equivalent launch test before publishing.

## Impact

- **Code**: `src/app/bootstrap.rs` (startup failure interception), new `src/app/renderer_fallback.rs` (bundled-stack discovery and self re-exec) and `src/app/startup_alert.rs` (stderr + Linux dialog chain), `src/i18n.rs` (startup-failure strings for all interface languages).
- **Packaging/CI**: new `scripts/build-lavapipe.sh` and `scripts/bundle-appimage-vulkan.sh`; `.github/workflows/release.yml` gains a cached lavapipe build, the bundling step and a launch smoke job; `scripts/verify-packaged-workspace.ps1` checks the bundled stack; `THIRD_PARTY_NOTICES.md` lists Mesa, LLVM, the Vulkan loader and the bundled X11 helper libraries.
- **Size**: the AppImage grows by roughly 20 MB compressed (≈58 MB lavapipe + loader uncompressed); glibc baseline stays at 2.35 (the stack itself needs ≤ 2.34).
- **Invariants**: none of the cached-per-version derived Markdown state, highlighting memoization or text-handle reuse is touched; the new code runs only before the first window opens.
- **Docs**: `docs/linux-gpu-troubleshooting.md` describes the fallback, the override variable and manual remediation.

## Non-goals

- Changing the renderer (no wgpu/OpenGL backend; that remains Phase B of `harden-linux-gpu-init`).
- Bundling Vulkan into the `.deb`/`.rpm` packages, or repairing host Vulkan installations.
- Native failure dialogs on Windows/macOS (still tracked by `harden-linux-gpu-init`; those platforms get the stderr message from this change).
