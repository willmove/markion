# Proposal: harden-linux-gpu-init

## Why

On GPU-less Linux machines (cloud desktops, VMs without GPU passthrough), Markion 0.3.8 dies at startup with a bare panic (`Unable to init GPU context: Platform(Init(ERROR_INITIALIZATION_FAILED))` from the vendored GPUI Wayland client) that gives users no clue what to do. Field debugging on an Omarchy cloud machine (2026-09-10) showed the situation is fixable: the app runs fine once the system Vulkan stack is sane, but Markion's current vendored GPUI is (a) Vulkan-only — no OpenGL fallback where only GL software rendering works, unlike current upstream Zed — and (b) silent about failures. We should make Markion start (or fail gracefully with guidance) on the same class of machines where current Zed runs.

## What Changes

- **Actionable GPU-init failure UX**: when the GPU renderer context cannot be initialized on any platform, Markion exits non-zero with (1) an OS-native, localized message dialog (GPUI cannot render, so no in-app UI is possible) and (2) a diagnostic log record. On Linux the message includes concrete remediation steps distilled from the field investigation: install a software Vulkan driver (`vulkan-swrast`/mesa), force it via `VK_ICD_FILENAMES`/`VK_LOADER_DRIVERS_SELECT`, disable a stale ICD manifest or a crashing implicit layer via `VK_LOADER_LAYERS_DISABLE`, and where to find the full troubleshooting doc.
- **Troubleshooting documentation**: ship `docs/linux-gpu-troubleshooting.md` capturing the three stacked root causes identified in the field (no software driver, stale NVIDIA ICD manifest, crashing Mesa `device_select` implicit layer) and their verified fixes.
- **Formalize software-Vulkan support**: pin the already-working behavior — Markion SHALL start and remain usable when the only Vulkan adapter is a software rasterizer (llvmpipe/lavapipe), matching the verified 0.3.8 field result.
- **Vendored GPUI/wgpu migration (phased)**: upgrade the vendored Zed crates to the wgpu-era GPUI so Linux gains the upstream multi-backend renderer (Vulkan → OpenGL software-rendering fallback) and the upstream emulated-GPU handling that lets current Zed run on these machines. This is a large API migration across Markion's UI code and is gated by a feasibility spike (see design).
- **BREAKING** (internal, not user-visible): the vendored `vendor/zed` snapshot and the root crate's renderer dependency set (`blade-graphics`/`blade-macros`/`blade-util` → wgpu stack) are replaced; all GPUI call sites in `src/` are ported to the new API.

## Capabilities

### New Capabilities

- `rendering-resilience`: Covers renderer-startup resilience — graceful, actionable, localized failure when the GPU context cannot initialize (OS-native dialog + diagnostics log, non-zero exit), and usable operation on machines whose only rendering path is software (software Vulkan adapter today; OpenGL fallback after the wgpu-era GPUI migration).

### Modified Capabilities

(none — the new capability stands alone; `chrome-platform`'s cross-platform requirement text is unchanged, this change adds rather than rewrites)

## Impact

- **Code**: `src/main.rs` (startup ordering: logging → renderer attempt → native failure dialog), new platform-diagnostics module for the native dialog path, `src/i18n.rs` (new localized strings), wholesale GPUI API migration across `src/` UI modules.
- **Dependencies**: vendored `vendor/zed` snapshot replaced with a current wgpu-era revision; root `Cargo.toml`/`Cargo.lock` renderer dependency swap; `[profile.dev.package]` overrides revisited if compute-heavy new deps land on the typing path.
- **Invariants touched**: none of the derived-state/highlighting/text-handle caching invariants change; the migration touches nearly every GPUI call site that renders them, so regression risk concentrates in rendering/timing behavior, not data flow. The `crates/*` members-never-depend-on-gpui rule is unaffected (vendored swap stays in the root crate).
- **CI/release**: Linux build must keep producing deb/AppImage artifacts; optional smoke test under `xvfb` + llvmpipe to prove the software-rendering path in CI. User-facing dialog strings go through `src/i18n.rs` per repo convention.

## Non-goals

- Fixing users' broken system Vulkan stacks automatically (detect and guide only — no silent driver reconfiguration).
- Any hardware-GPU feature work, renderer performance tuning, or changing Markion's visual output.
- Windows/macOS behavioral changes beyond sharing the same graceful GPU-init failure dialog.
- Porting Zed features that ride along with newer GPUI revisions beyond what Markion's UI needs to compile and behave equivalently.
