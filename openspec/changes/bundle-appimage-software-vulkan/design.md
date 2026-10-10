# Design: bundle-appimage-software-vulkan

## Context

See proposal.md for motivation. Constraints that shape the approach:

- GPUI's Linux renderer is blade-graphics 0.7 (Vulkan only). `X11Client::new()` creates the GPU context before any window and `current_platform()` unwraps the error; the Wayland client `expect`s it. Both panic inside `Application::new()`, before Markion code runs. Release builds keep `panic = "unwind"`.
- blade loads `libvulkan.so.1` with `dlopen` at context creation, so the Vulkan loader, its driver search (`VK_DRIVER_FILES`) and its library search (`LD_LIBRARY_PATH`) are read at that point. `LD_LIBRARY_PATH` changes only take effect in a new process image.
- The AppImage is produced by cargo-packager 0.11.8 (AppImageKit ELF `AppRun` that execs `usr/bin/markion`), then extracted, permission-normalized and repacked by `scripts/fix-appimage-permissions.sh` (PR #17), then signed in `prepare-update`. It references glibc ≤ 2.35 because it is built on ubuntu-22.04.
- Verified manually on 2026-10-10 in an ubuntu-22.04 container without `mesa-vulkan-drivers`: v0.4.8 reproduces the catalog panic; the same binary with `VK_DRIVER_FILES` pointing at a lavapipe built by this design opens and fully renders its window under Xvfb + icewm (`Adapter: "llvmpipe (LLVM 15.0.7, 256 bits)"`).

## Goals / Non-Goals

**Goals:**

- Zero cost on hosts with working Vulkan: no probe, no extra process, no bundled library loaded.
- One fallback attempt, decided by the real GPUI initialization rather than a separate heuristic probe that could disagree with blade's device selection.
- Keep the AppImage glibc baseline and the PR #17 permission and signing order intact.

**Non-Goals:**

- Detecting hosts whose Vulkan driver *crashes* (SIGSEGV) during initialization; those cannot be caught in-process. `MARKION_SOFTWARE_VULKAN=1` is the documented escape hatch.
- Shipping the software stack in `.deb`/`.rpm` (system packages should depend on the distribution's driver).

## Decisions

### D1 — Build lavapipe from source with static LLVM (not the distribution package)

`scripts/build-lavapipe.sh` builds Mesa 24.3.4 with only `-Dvulkan-drivers=swrast`, `-Dshared-llvm=disabled` against Ubuntu 22.04's `llvm-15-dev`, and the Khronos Vulkan loader v1.3.296, on ubuntu-22.04. Result: one ≈58 MB `libvulkan_lvp.so` (≈20 MB gzip) whose only non-trivial dependencies are X11/xcb helpers, plus a 0.5 MB loader; highest glibc symbol 2.34.

- Alternative rejected: copying jammy's `mesa-vulkan-drivers` package. Its `libvulkan_lvp.so` links `libLLVM-15.so.1` (≈100 MB), which drags in `libxml2.so.2` → `libicu70` (≈30 MB); newer distributions moved to `libxml2.so.16`, so these would all have to be bundled, roughly doubling the size and mixing a second LLVM into any process that also loads the host's Mesa.
- Alternative rejected: requiring `mesa-vulkan-drivers` on the host (status quo; fails the catalog and the stated goal).
- Ubuntu's static LLVM names `-lPolly -lPollyISL` for the `lto` component but ships Polly only as a plugin; nothing lavapipe uses references Polly, so empty stub archives satisfy the link.
- Mesa's tarball is pinned by SHA-256; the CI step caches the output keyed on the script hash so the ≈6-minute build runs only when the script changes.

### D2 — Library closure follows the AppImage excludelist

Dependencies on the AppImage excludelist (glibc, libstdc++, libgcc_s, zlib, libdrm, libxcb, libX11-xcb, libxcb-dri3, libwayland-client) come from the host. The remaining small libraries (`libtinfo.so.6`, `libxcb-{randr,present,xfixes,sync,shm}`, `libxshmfence`) are copied next to the ICD with their copyright files. The build script fails on any unclassified `NEEDED` entry, so a Mesa bump cannot silently add a host dependency. These libraries live in `usr/lib/markion-vulkan/`, which is only put on `LD_LIBRARY_PATH` in fallback mode, so they never shadow host libraries on hardware-rendering hosts.

### D3 — Reactive fallback by re-executing the binary (not an AppRun probe)

`run_with_startup_intent` wraps `Application::new().run(..)` in `catch_unwind` (the mechanism already chosen by `harden-linux-gpu-init` D1). A panic before the first window has finished initializing is classified; a renderer failure (`Unable to init GPU context`, failed main-window open) triggers `renderer_fallback`, which — if `usr/lib/markion-vulkan/lvp_icd.x86_64.json` exists next to the running executable, the fallback is not disabled and not already active — `exec`s `/proc/self/exe` with the original arguments and:

- `VK_DRIVER_FILES` and `VK_ICD_FILENAMES` → the bundled manifest (whose `library_path` is manifest-relative, so any mount point works),
- `VK_LOADER_LAYERS_DISABLE=~implicit~` and `NODEVICE_SELECT=1` → host implicit layers (e.g. a crashing Mesa `device_select`) are not injected into the fallback,
- `LD_LIBRARY_PATH` prefixed with the bundled directory → the bundled loader and helper libraries are used,
- `MARKION_SOFTWARE_VULKAN_ACTIVE=1` → the new process never re-execs again and logs that the software stack is active.

`exec` keeps the PID, so the AppImage runtime (FUSE mount or firejail sandbox) stays alive and `APPIMAGE`/`APPDIR` stay valid for the updater. Panics after startup completed are resumed unchanged.

- Alternative rejected: a shell `AppRun` that probes Vulkan before launching. It needs a probe program, costs a Vulkan instance/device creation on every launch even with a working GPU, can disagree with blade's own device filtering, and replaces cargo-packager's AppRun.
- Alternative rejected: retrying `Application::new()` in-process after changing `VK_DRIVER_FILES`. `LD_LIBRARY_PATH` cannot change for the running process, and GPUI's partially initialized platform state after a panic is not safe to reuse.
- Alternative rejected: making GPUI's platform constructor fallible in the vendored tree; `harden-linux-gpu-init` D1 already rejected that for being invasive.

### D4 — Failure report: stderr always, native dialog when a tool exists

`startup_alert` composes a localized title/body from `src/i18n.rs` (language from the persisted `config.toml` preference, English otherwise — the same resolution the app uses), always writes it to stderr, then on Linux tries `zenity` → `kdialog` → `xmessage` when `DISPLAY`/`WAYLAND_DISPLAY` is set. The command chain is built by a pure function and run through an injectable spawner so ordering and arguments are unit-tested. Windows/macOS currently get the stderr text only (their native dialogs remain `harden-linux-gpu-init` task 1.2). `MARKION_SIMULATE_RENDERER_FAILURE=1` panics with the canonical GPU-context message before `Application::new()` so the path is testable without breaking Vulkan.

### D5 — Packaging order and verification

`release.yml` (Linux job): build/restore lavapipe → `cargo packager` → `scripts/bundle-appimage-vulkan.sh dist/*.AppImage <stack>` (extract, copy into `usr/lib/markion-vulkan/`, repack) → `scripts/fix-appimage-permissions.sh` (unchanged, PR #17) → `verify-packaged-workspace.ps1` (additionally asserts the bundled stack files) → upload. A new `appimage-smoke` job on ubuntu-22.04 installs the catalog's GUI packages and its firejail build, asserts no Vulkan ICD is present, launches the artifact with `firejail --appimage`, waits for `xdotool search --onlyvisible`, and uploads the screenshot. `release` and `mirror-oss` depend on it. Signing in `prepare-update` still runs on the downloaded final artifact, so `.sig`/`update.json` cover the bundled stack.

## Risks / Trade-offs

- [AppImage grows ≈20 MB compressed] → Accepted for out-of-the-box startup; the stack is one file set that can be dropped if GPUI gains a GL fallback (Phase B of `harden-linux-gpu-init`).
- [Software rendering is slow on large windows] → Expected for a fallback; logged at startup so bug reports show it, documented in the troubleshooting doc.
- [A host Vulkan driver that segfaults cannot be caught] → `MARKION_SOFTWARE_VULKAN=1` documented; implicit host layers are disabled in fallback mode.
- [Panic payload text drifts with a GPUI update] → Matched on stable substrings shared with `harden-linux-gpu-init`; the simulation hook pins the canonical message in tests.
- [Bundled helper libraries older than the host's] → Only on `LD_LIBRARY_PATH` in fallback mode; they are ABI-stable xcb extension libraries.
- [Mesa download unavailable in CI] → Output cached; source pinned by checksum; failure blocks the build instead of shipping without the stack.

## Migration Plan

Purely additive: no persisted format changes. Rollback = revert the PR; the AppImage returns to requiring a host Vulkan driver.
