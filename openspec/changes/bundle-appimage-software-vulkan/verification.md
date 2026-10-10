# Verification: bundle-appimage-software-vulkan

All runs used the AppImage built from this branch with the release pipeline's
Linux steps, in order: `cargo packager --formats appimage` →
`scripts/bundle-appimage-vulkan.sh` → `scripts/fix-appimage-permissions.sh`.
The build ran in an `ubuntu:22.04` container with the same apt packages as the
release job. Result: `markion_0.4.8_x86_64.AppImage`, 52,954,304 bytes,
sha256 `fdbaee1c718e653d80a9cc82e718438450d9e8838185b00a92deb4fc62d9dcc3`.

## Test environment (mirrors the AppImage catalog test)

The catalog test is `code/worker.sh` / `wf-test.yml` in AppImage/appimage.github.io.
The container reproduces it:

- `ubuntu:22.04`, with the catalog's GUI and GL packages installed (`xvfb icewm
  x11-utils xdotool imagemagick libgl1-mesa-dri libosmesa6 mesa-utils libfuse2
  fuse tesseract-ocr`). `mesa-vulkan-drivers` is **not** installed, so there is no
  Vulkan ICD at all.
- The catalog's firejail build (`alpine-firejail-git20230825.tar.gz` plus musl),
  installed setuid exactly like `worker.sh` does.
- Run as a regular user: Xvfb `:99` at 800x600x24, then icewm, then
  `firejail --quiet --noprofile --net=none --appimage ./Markion.AppImage`.
  After that, `xdotool search --onlyvisible`, the catalog's own
  `take-screenshot.sh` and `check-screenshot.sh` (blank-window and OCR
  error-text checks), and the "still running when killed" success criterion.
- Two variants:
  - **with loader**: the system `libvulkan1` 1.3.204 is installed. This is the
    exact catalog failure: `Instance extension "VK_KHR_surface" is not supported`.
  - **without loader**: there is no `libvulkan.so.1` on the system.

## 1.1 Stack build

`scripts/build-lavapipe.sh` on ubuntu-22.04 produces:

- `libvulkan_lvp.so` (Mesa 24.3.4, LLVM 15.0.7 linked statically)
- `libvulkan.so.1` (loader 1.3.296)
- `lvp_icd.x86_64.json` (`"library_path": "./libvulkan_lvp.so"`)
- the bundled libraries `libtinfo.so.6`, `libxcb-{present,randr,shm,sync,xfixes}`
  and `libxshmfence.so.1`
- `licenses/` (10 files) and `VERSIONS`

No NEEDED entry falls outside the host-excludelist set or the bundled set.
The catalog's `check-libc.sh` reports `X-AppImage-Glibc-Required=GLIBC_2.35`
both before and after bundling, so the baseline is unchanged. It reports
`X-AppImage-Runtime=dynamic`, the same as the released v0.4.8 (that comes from
the PR #17 appimagetool repack, not from this change).

## 1.2 / 3.1 Bundling and permissions

- The extracted AppImage contains `usr/lib/markion-vulkan/` with the stack.
- Permissions after the unchanged `fix-appimage-permissions.sh`: AppRun 755,
  `usr/bin/markion` 755, every directory 755, and the libraries 644.
- The catalog's `appdir-lint.sh` reports "Lint found no fatal issues".

Size, measured on the same build:

| Build | Bytes |
| --- | --- |
| Without the stack (permission fix only) | 32,687,296 |
| With the stack | 52,954,304 (+20,267,008, +62%) |
| Released v0.4.8, for reference | 33,207,488 |

## 3.2 Packaged-artifact verification

`scripts/verify-packaged-workspace.ps1 -Format appimage` was run with
PowerShell 7.4 in the build container:

- On the final AppImage it passes and prints `AppImage software Vulkan stack OK:
  mesa=24.3.4, llvm=15.0.7, vulkan-loader=1.3.296`.
- On the same build without the stack (permission-fixed) it fails with
  `AppImage software Vulkan stack is missing usr/lib/markion-vulkan/...`.

## 4.1 Catalog-equivalent runs (no Vulkan driver)

| Variant | Window | Catalog screenshot check | Result |
| --- | --- | --- | --- |
| v0.4.8 release, with loader | none: exits with the `VK_KHR_surface` / `NoSupportedDeviceFound` panic | n/a | fail (reproduces the catalog) |
| this branch, with loader | `Markion - Untitled.md` visible, full UI rendered | 90% one color, 56 words, no error text | `RESULT=0`, still running when killed |
| this branch, without loader | `Markion - Untitled.md` visible, full UI rendered | 90% one color, 56 words, no error text | `RESULT=0`, still running when killed |

The log shows the intended sequence:

1. The host stack fails.
2. `restarting Markion on the bundled software Vulkan driver (Mesa lavapipe)
   stack=/tmp/.mount_…/usr/lib/markion-vulkan`.
3. `rendering with the bundled software Vulkan driver`.
4. `Adapter: "llvmpipe (LLVM 15.0.7, 256 bits)"`.

The re-exec keeps the process, so the AppImage mount and the firejail sandbox
stay alive.

`scripts/smoke-test-appimage.sh` (the CI `appimage-smoke` job script) was run
in the same container with `MARKION_SMOKE_REQUIRE_NO_VULKAN_DRIVER=1`:

- It passes for this branch.
- It fails for v0.4.8 with "the application exited instead of showing a window".
- It refuses to run in a container that has `mesa-vulkan-drivers` installed.

## 3.3 Pull-request CI (GitHub-hosted ubuntu-22.04)

The `release` workflow passed on PR #18 in runs 38047584887 and 38049146341.

- The first run built the stack from scratch ("Build software Vulkan stack",
  about 2 minutes 45 seconds) and saved it to the cache. The second run
  restored it from the cache and skipped the build.
- `appimage-smoke` ran with all system ICD manifests moved away (`none`) and
  the catalog's firejail 0.9.73, which mounts the image at
  `/run/firejail/appimage`. Its log shows:
  - `Instance extension "VK_KHR_surface" is not supported`
  - `restarting Markion on the bundled software Vulkan driver …
    stack=/run/firejail/appimage/usr/lib/markion-vulkan`
  - `Adapter: "llvmpipe (LLVM 15.0.7, 256 bits)"`
  - `Markion - Untitled.md` visible
  - `PASS`
- The screenshot is uploaded as the `appimage-smoke-test` artifact.
- `Verify packaged MarkNice workspace (deb,appimage,rpm)` passed with the new
  stack assertion.

The `quality` workflow fails on `clippy::approx_constant` in
`src/app/preview.rs`. `main` fails the same way since Rust 1.99
(run 38037606920), and clippy reports nothing in the files this change adds.

## 4.2 Hardware-first path and overrides

With `mesa-vulkan-drivers` 23.2.1 installed, the system lavapipe stands in for
a GPU driver. The AppImage starts without restarting.
`/proc/<pid>/maps` shows only `/usr/lib/x86_64-linux-gnu/libvulkan.so.1.3.204`
and the system ICDs. Nothing from `usr/lib/markion-vulkan` is mapped, and
`MARKION_SOFTWARE_VULKAN_ACTIVE` is not set.

Overrides, run under firejail in the catalog environment:

| Scenario | Result |
| --- | --- |
| default | restarts once on the bundled driver; window shown |
| `MARKION_SOFTWARE_VULKAN=1` | restarts on the bundled driver before trying the host stack; window shown |
| `MARKION_SOFTWARE_VULKAN=0` | no fallback (`reason=Disabled`); remediation text on stderr; xmessage dialog; exit status 1 after Enter |
| `MARKION_SIMULATE_RENDERER_FAILURE=1` | fails, restarts on the bundled driver, fails again; the alert adds "The software Vulkan driver bundled with Markion could not start either."; exit status 1; no third attempt |
| `MARKION_SIMULATE_RENDERER_FAILURE=1 MARKION_SOFTWARE_VULKAN=0` | alert without a fallback attempt; exit status 1 |

## 4.3 Automated checks

- `cargo test --workspace --no-fail-fast` in the ubuntu-22.04 container:
  - `markion` lib: 681 passed. `markion` bin: 688 passed, including the 12 new
    `startup_alert` / `renderer_fallback` tests.
  - The i18n catalog test passed.
  - All other crates passed except three font-dependent tests, in crates this
    change does not touch, that fail in a container without fonts:
    - `markdown` `math::tests::representative_corpus_matches_expected_outcomes`
      passes once a CJK font is installed.
    - `markion-pdf` `spike_svg_with_text_element` / `spike_pdf_api_surface`
      need the font set those spikes were written against.
  - The vendored `vendor/cc-rs` keeps `#![deny(warnings)]` and does not build
    on Linux with Rust 1.99 (`fetch_update` deprecation), so tests ran with
    `RUSTFLAGS="--cap-lints warn"`. The release build ran with
    `RUSTFLAGS="-D warnings"` and passed.
- `cargo fmt --all -- --check`: clean.
- `openspec validate bundle-appimage-software-vulkan --strict`: valid.
