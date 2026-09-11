# Design: harden-linux-gpu-init

## Context

Markion renders through a vendored snapshot of Zed's GPUI. On Linux that snapshot initializes a Vulkan-only renderer via blade-graphics 0.7.1: `vendor/zed/crates/gpui/src/platform/linux/wayland/client.rs:501` does `BladeContext::new().expect("Unable to init GPU context")`, so a machine with a broken or absent Vulkan stack kills the process with a bare panic before any Markion UI exists. The field investigation (Omarchy cloud machine, 2026-09-10) established three facts that shape this design:

1. The vendored blade stack runs fine on a software Vulkan adapter (llvmpipe) once the system Vulkan layer is sane — no application change needed for that path, only regression protection.
2. Current upstream Zed runs on the same machine with zero working Vulkan, implying its wgpu-era renderer has a non-Vulkan (OpenGL) path that Markion's snapshot lacks.
3. All user-visible failures today are a panic string on stderr; the fixes (install `vulkan-swrast`, force the ICD via loader env vars, disable a stale NVIDIA manifest or the crashing Mesa `device_select` implicit layer) are documentable and stable.

Startup order today (`src/app/bootstrap.rs::run_with_startup_intent`): `init_logging()` → highlighter warm-up thread → `Application::new().run(closure)`; the language preference is loaded inside the closure (`MarkionApp::new`), i.e. **after** the point where GPU init can panic. Release builds keep the default `unwind` panic strategy (`Cargo.toml` `[profile.release]`), and the same file pins GPUI 0.2.2.

## Goals / Non-Goals

**Goals:**

- Turn any renderer-startup failure into an actionable, localized, OS-native message with a non-zero exit — without requiring Markion's own renderer to work.
- Make software-Vulkan startup an explicitly protected behavior (spec + verification), not an accident.
- Get Linux an OpenGL software-rendering fallback by moving the vendored GPUI snapshot to the wgpu era, behind a feasibility gate that can split the work into a dedicated change if needed.
- Keep the failure path out of the typing/rendering hot path entirely.

**Non-Goals (design-level):**

- Auto-repairing users' Vulkan configuration (no writing to `/usr/share/vulkan`, no driver installs).
- In-app (GPUI-rendered) error UI for this failure class — impossible by definition; the native alert is the ceiling.
- Performance work for software rendering beyond expectation-setting in docs.
- Pulling in unrelated newer-GPUI features during the migration beyond compile/run parity.

## Decisions

### D1 — Phase A intercepts the panic with `catch_unwind`, not vendored surgery

Wrap `Application::new().run(...)` in `std::panic::catch_unwind(AssertUnwindSafe(...))` in `src/app/bootstrap.rs`. On `Err`, inspect the payload string: payloads matching the renderer failures (`"Unable to init GPU context"`, window-open failure) get the localized GPU remediation message; anything else gets a generic "Markion failed to start" message that still names the panic text and the log directory.

- Why: zero changes to the vendored tree, platform-agnostic (covers Windows/macOS GPU failures too), and viable because release keeps `unwind`. The payload is available from `catch_unwind`'s `Err` directly, so no fragile panic-hook games are needed.
- Alternative rejected: patching `WaylandClient::new` to return `Result` and threading it through GPUI's platform-init API — invasive across the vendored crate for one message; reconsidered only if some platform turns out to abort instead of unwinding.

### D2 — Native alert is a small hand-rolled module with a fallback chain

New `src/app/startup_alert.rs`: Windows `MessageBoxW` via the `windows` crate already in the dependency tree (gpui pulls it); macOS `NSAlert` run-modal via the objc runtime gpui already links; Linux spawns `zenity` → `kdialog` → `xmessage` in order. If every mechanism fails, print the same text to stderr and still exit non-zero (spec scenario). The dialog is fire-and-forget synchronous; no event loop is required.

- Why: the dialog must work precisely in degraded, GPU-less environments; a hand-rolled chain adds no new dependency and no toolkit assumptions, while crates like `rfd` drag GTK into a context where we cannot assume a working GPU stack.
- Alternative rejected: `native-dialog` crate (similar shell-out behavior but a new dependency for ~100 lines we can own and test).

### D3 — Failure-path localization reads the language preference directly

The i18n catalog lives in the gpui-free `markion` lib crate and initializes without a window. At failure time, read the persisted `language` field from `config.toml` via the existing storage loader (cheap, no gpui), fall back to the OS locale, then to English. This sidesteps the fact that the normal in-app language load happens after GPU init.

### D4 — Test hook: `MARKION_SIMULATE_RENDERER_FAILURE=1`

When set, `run_with_startup_intent` deliberately panics with the canonical GPU-context message before `Application::new()`. This makes the failure UX verifiable on Windows (primary dev platform) and in CI on all three OSes without breaking real GPU stacks. Documented in the troubleshooting doc; honored in all builds.

### D5 — Troubleshooting doc is the single source for remediation text

`docs/linux-gpu-troubleshooting.md` captures the three stacked root causes from the field (no software driver → stale NVIDIA ICD manifest → crashing Mesa `device_select` implicit layer) with verified commands. The dialog's short steps and the doc must stay consistent (spec scenario); the doc also records the AppImage-specific note that Markion bundles no Vulkan libraries and uses the host stack.

### D6 — Phase B (wgpu migration) is gated by a spike and may split out

Phase B replaces the vendored `vendor/zed` snapshot with a current wgpu-era revision and ports all GPUI call sites in `src/`. Before any porting, a time-boxed spike on a throwaway branch swaps the dependency and inventories the compile-error surface. The spike produces a written go/no-go in this change's `verification.md`:

- **Go**: port proceeds inside this change on branch `feature/gpui-wgpu-migration`, started only after `add-git-workspace-sync` is archived (avoiding a mega-merge with the in-flight `feature/git-sync` work).
- **No-go (scope too large)**: Phase B is extracted into a dedicated change; this change archives with Phase A delivered plus the spike report, and the `rendering-resilience` spec's OpenGL-fallback requirement moves to the new change. This split requires updating the proposal/specs of this change accordingly — it is a recorded decision, not silent scope loss.

Vendored-fork policy after the swap: keep upstream's `ZED_DEVICE_ID` handling; flip the emulated-GPU default so software adapters are accepted **without** an env-var opt-in (spec requirement), while still accepting `ZED_ALLOW_EMULATED_GPU=1` as a harmless no-op for muscle-memory parity with Zed. The GL-fallback claim itself must be confirmed in upstream source during the spike (see Risks).

### D7 — Renderer data flow and caching are untouched by design

Document edit → version bump → derived Markdown state computed once per version → `Arc`-shared consumers → editor element paint → GPUI → backend (blade today, wgpu after Phase B). Phases A and B only touch the last arrow and the pre-window startup path: no change to cache identity, memoization keys, text-handle reuse per version, or undo snapshots. The risk surface of Phase B is frame scheduling/paint behavior, covered by regression verification rather than data-flow redesign.

## Risks / Trade-offs

- [A platform aborts instead of unwinding, so `catch_unwind` never sees the failure] → Release keeps `unwind`; the D4 hook lets us verify the path per-OS. If a platform is later found to abort, fall back to D1's rejected vendored-`Result` patch for that platform only.
- [Minimal Linux has none of zenity/kdialog/xmessage] → stderr fallback is spec-sanctioned and always present; exit code stays non-zero.
- [Panic payload matching drifts if the vendored panic text changes] → Match on stable substrings; the D4 hook pins the canonical message in tests; Phase B replaces the text anyway and the matcher is updated in the same PR.
- [Upstream wgpu GPUI turns out to have no real GL fallback on Linux] → Spike gate: revise this change's spec before any Phase B implementation (the requirement would become Vulkan-software-only plus documented guidance). Recorded as a gate, not an assumption.
- [Phase B compile-error surface exceeds a reasonable PR series] → D6 no-go path extracts it to a dedicated change.
- [Software-rendering performance complaints] → Doc sets expectations; functionality-over-smoothness is the spec'd contract.
- [wgpu dependency tree grows build time] → Accepted; revisit `[profile.dev.package]` overrides per the crate-architecture spec if the typing path feels it.

## Migration Plan

1. Phase A lands as one PR (startup alert module, bootstrap wiring, i18n keys, doc, test hook + tests). Purely additive at startup; revert with a single `git revert`. No persisted-format changes.
2. Phase B (post-spike-go): branch series — vendor swap PR (compiles, no behavior claims), then port PRs grouped by UI module, each keeping `cargo test --workspace` green. Rollback = drop the branch; main never carries a half-ported tree.
3. Release note: Phase A is releasable on the next patch version as soon as its tasks complete, even while this change folder remains open for Phase B (releases cut from `main`, not from change-archive state).

## Open Questions

- Exact upstream Zed revision to vendor (resolved at spike time; preference for a tagged Zed release for traceability).
- Linux alert command order (`zenity` vs `kdialog` first) — implementation detail, safely deferrable.
