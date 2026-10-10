# Tasks

## 1. Bundled software Vulkan stack

- [ ] 1.1 Add `scripts/build-lavapipe.sh` (pinned Mesa lavapipe with static LLVM + Vulkan loader, excludelist-based library closure, license texts) and verify on ubuntu-22.04 that it produces `libvulkan_lvp.so`, `libvulkan.so.1`, `lvp_icd.x86_64.json` with no glibc symbol newer than 2.35.
- [ ] 1.2 Add `scripts/bundle-appimage-vulkan.sh` that copies the stack into `usr/lib/markion-vulkan/` of an AppImage and repacks it; verify by extracting the result and listing the directory.
- [ ] 1.3 List Mesa, LLVM, the Vulkan loader and the bundled X11/terminfo libraries in `THIRD_PARTY_NOTICES.md`; verify every file in the stack's `licenses/` directory is referenced.

## 2. Startup failure handling and software fallback

- [ ] 2.1 Add startup-failure strings (title, Linux renderer remediation, generic renderer, software-fallback-failed note, generic startup failure, log-directory line) to `src/i18n.rs` for every interface language; verify with an i18n unit test that every language has non-empty, placeholder-complete text.
- [ ] 2.2 Add `src/app/startup_alert.rs` (stderr + Linux `zenity` → `kdialog` → `xmessage` chain through an injectable spawner); verify with unit tests for command construction, fallback ordering and the no-display case.
- [ ] 2.3 Add `src/app/renderer_fallback.rs` (bundled-stack discovery relative to the executable, `MARKION_SOFTWARE_VULKAN` modes, fallback environment, one-shot re-exec); verify with unit tests for discovery, mode parsing and the composed environment.
- [ ] 2.4 Wire `src/app/bootstrap.rs`: `catch_unwind` around startup, panic classification, fallback re-exec, alert + log + non-zero exit, resume of post-startup panics, `MARKION_SIMULATE_RENDERER_FAILURE=1` hook, forced mode, and a log line when the software stack is active; verify with unit tests for classification and with `MARKION_SIMULATE_RENDERER_FAILURE=1` producing the alert text on stderr and exit status 1.
- [ ] 2.5 Write `docs/linux-gpu-troubleshooting.md` (bundled fallback, override variable, manual remediation consistent with the dialog text); verify the steps match the English strings from 2.1.

## 3. Release pipeline

- [ ] 3.1 Update `.github/workflows/release.yml`: cached lavapipe build, bundling before `fix-appimage-permissions.sh`, unchanged signing order; verify the workflow YAML parses and the Linux steps run in that order.
- [ ] 3.2 Extend `scripts/verify-packaged-workspace.ps1` to assert the bundled stack files in the AppImage; verify against a locally built AppImage.
- [ ] 3.3 Add the `appimage-smoke` job (ubuntu-22.04, Xvfb + icewm, catalog firejail build, no Vulkan ICD, `firejail --appimage`, `xdotool search --onlyvisible`, screenshot artifact) and make `release`/`mirror-oss` depend on it; verify it passes on the pull-request run.

## 4. Integration verification

- [ ] 4.1 In a clean ubuntu-22.04 container without `mesa-vulkan-drivers`, run the built AppImage through `firejail --appimage` under Xvfb + icewm (with and without a system Vulkan loader) and record that a visible window appears, with screenshot, in `verification.md`.
- [ ] 4.2 Verify the hardware-first path: with a host Vulkan driver present the bundled stack is not loaded, and `MARKION_SOFTWARE_VULKAN=0` produces the alert instead of the fallback; record in `verification.md`.
- [ ] 4.3 Run `cargo test --workspace` and `openspec validate bundle-appimage-software-vulkan`.

## Workflow follow-up

- Archive the change after review and after a tagged release has passed the AppImage catalog retest (AppImage/appimage.github.io#6581).
