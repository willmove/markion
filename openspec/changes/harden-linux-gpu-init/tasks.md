## 1. Phase A — renderer-failure alert and diagnostics (shippable standalone)

- [ ] 1.1 Add failure-dialog string keys to `src/i18n.rs` (renderer-failure title, Linux remediation body, generic startup-failure body, log-directory line) with translations for every supported interface language.
- [ ] 1.2 Create `src/app/startup_alert.rs`: synchronous OS-native alert — Windows `MessageBoxW` (existing `windows` dep), macOS `NSAlert` via the objc runtime gpui already links, Linux `zenity` → `kdialog` → `xmessage` spawn chain — with a guaranteed stderr fallback that never panics; unit-test command construction and fallback ordering with an injectable spawner.
- [ ] 1.3 Implement failure-path language resolution: read the persisted `language` from `config.toml` via the existing gpui-free storage loader, fall back to OS locale, then English; expose it to the alert composer.
- [ ] 1.4 Wire `src/app/bootstrap.rs`: wrap `Application::new().run(...)` in `std::panic::catch_unwind`, classify the payload (renderer-failure substrings vs generic), emit a `tracing::error!` record with the payload, show the localized alert, exit non-zero.
- [ ] 1.5 Add the `MARKION_SIMULATE_RENDERER_FAILURE=1` test hook (panics with the canonical GPU-context message before `Application::new()`) and an integration test asserting the hook produces the renderer-failure classification.
- [ ] 1.6 Write `docs/linux-gpu-troubleshooting.md` covering the three field-verified root causes (no software Vulkan driver; stale NVIDIA ICD manifest; crashing Mesa `device_select` implicit layer), their fixes, the AppImage note (no bundled Vulkan libs), and the test hook — keeping the steps consistent with the dialog text (spec scenario).
- [ ] 1.7 Run `cargo test --workspace` and manually verify the alert on Windows via the test hook (dialog shown, log record written, non-zero exit).

## 2. Software-Vulkan regression protection

- [ ] 2.1 Add a Linux CI smoke job (or scripted local workflow if runner constraints bite): `xvfb` + Mesa lavapipe, launch the release binary with the loader forced to the software ICD, assert the process stays alive past window creation and exits cleanly on SIGTERM.
- [ ] 2.2 Record the software-Vulkan verification evidence in `verification.md` (manual cloud-machine run from 2026-09-10 counts once Phase A ships in a release containing it).

## 3. Phase B gate — wgpu feasibility spike

- [ ] 3.1 On a throwaway branch, swap the vendored `vendor/zed` snapshot for a current wgpu-era Zed revision and inventory the compile-error surface across `src/` (time-boxed; no porting).
- [ ] 3.2 Confirm from upstream source whether the Linux wgpu renderer actually falls back to OpenGL when Vulkan cannot initialize, and note the revision and evidence.
- [ ] 3.3 Write the go/no-go assessment into `verification.md`. On no-go (scope or missing GL fallback), split Phase B into a dedicated change with the user, and shrink this change's `rendering-resilience` spec accordingly before proceeding to group 4.

## 4. Phase B — vendored GPUI wgpu migration (only on a go decision)

- [ ] 4.1 Refresh the vendored crate set and swap the root renderer dependencies (`blade-graphics`/`blade-macros`/`blade-util` → the wgpu stack) so the workspace compiles and links; keep `[profile.dev.package]` overrides per the crate-architecture spec.
- [ ] 4.2 Port `src/` GPUI call sites to the new API in grouped, individually-testable commits (bootstrap/state; editor element and text system; layout/menus/shortcuts; preview and render surfaces; dialogs/overlays), keeping `cargo test --workspace` green after each group and preserving the cached-per-version derived-state invariants (no cache identity, memoization key, or text-handle reuse changes).
- [ ] 4.3 Apply the vendored-fork policy patches: accept software/emulated Vulkan adapters by default without an env-var opt-in, keep `ZED_DEVICE_ID` handling, accept `ZED_ALLOW_EMULATED_GPU=1` as a harmless no-op.
- [ ] 4.4 Update the Phase A panic-payload matcher if the vendored failure text changed, and re-verify the alert path end-to-end on the new stack via the test hook.
- [ ] 4.5 Verify on Linux: software-Vulkan smoke (group 2 job) passes, and with all Vulkan ICDs masked the binary still starts via the GL fallback, logging the fallback activation (spec scenario).
- [ ] 4.6 Run a typing-path performance sanity check on a hardware-GPU machine against the pre-migration baseline; investigate regressions before merge.

## 5. Closeout

- [ ] 5.1 Run `openspec validate harden-linux-gpu-init` and fill `verification.md` with per-requirement evidence.
- [ ] 5.2 Confirm release readiness: Phase A rides the next patch release from `main`; flag that Phase B warrants at least a minor version bump and updated Linux system-requirements notes.
