## 1. Runtime Provenance and Size Gate Spike

- [x] 1.1 Record the immutable pre-change Git revision and add a machine-readable PDF size-budget manifest defining 6 MiB per packaged artifact, 10 MiB per installed/staged payload, byte units, supported targets/formats, and the same-environment control-build rule.
- [x] 1.2 Add a PowerShell 7 PDFium acquisition/staging script plus unit-testable manifest parsing that selects only the current target's pinned non-V8/non-XFA PDFium 7881 archive member, verifies SHA-256 and expected size, rejects traversal/unexpected members, and writes only beneath an ignored `target/` directory.
- [x] 1.3 Scaffold the GUI-free `markion-pdf-viewer` workspace member with the minimal explicitly-featured `pdfium-render` binding and runtime-load probe; keep `gpui`, WebView/PDF.js, bundled fonts, V8/XFA, headers, import libraries, samples, and debug artifacts out of its dependency and package trees.
- [x] 1.4 Add cross-platform package/payload measurement tooling and tests that emit control/candidate/delta bytes plus largest changed files, use final artifact lengths and format-native staged/installed trees, fail on missing/unmeasurable inputs, and never mutate the configured budgets.
- [x] 1.5 Wire a non-publishing native CI size-spike matrix for Windows NSIS, macOS app/DMG, Linux DEB, and Linux AppImage; retain each JSON report and stop this change before application integration unless every minimal candidate is within both hard budgets.

## 2. GPUI-Free PDF Renderer Core

- [x] 2.1 Define pure request/result types for document IDs, generations, page geometry, raw RGBA rasters, safe error categories, and the 512 MiB source / 10,000-page / 32 MiB raster limits; expose no native handles across the crate boundary.
- [x] 2.2 Implement deterministic runtime discovery that uses the packaged target-specific resource path in release builds, permits only an explicit developer override in non-release builds, and never downloads or silently binds an arbitrary system/current-directory library at application startup.
- [x] 2.3 Implement document open and metadata extraction on the renderer owner, including case-independent path input, finite file/page checks, natural page dimensions, encrypted/password-required detection, and safe mapping of native diagnostics.
- [x] 2.4 Implement page rasterization to owned RGBA bytes with target-width bucketing, aspect preservation, pre-allocation pixel/byte reduction, and page-scoped render errors; avoid enabling broad image decoder features.
- [x] 2.5 Implement the bounded single-owner service thread and command/event protocol with one native call at a time, bounded/coalesced request queues, visible-range priority, close/shutdown handling, and stale `(document_id, generation)` rejection.
- [x] 2.6 Add a fake renderer backend plus headless tests for open/render/close ordering, serialization, queue coalescing, stale-result rejection, file/page/raster limits, encrypted/corrupt classification, and clean shutdown without requiring PDFium or GUI libraries.
- [x] 2.7 Add small non-packaged PDF fixtures and staged-runtime integration tests covering one page, mixed page sizes, multiple pages, embedded fonts, a corrupt file, a password-protected file, page render failure handling, and runtime-missing/digest-mismatch behavior.

## 3. Supported-Path and Heterogeneous Tab Integration

- [ ] 3.1 Introduce one shared `SupportedPathKind::{Document, Image, Pdf}` classifier, add case-insensitive `.pdf` and a PDF icon/file-tree kind, and test every existing document/image extension plus PDF and unsupported binary paths.
- [ ] 3.2 Extend `WorkspaceTab` with `PdfTabState` containing only normalized path, renderer identities, immutable page geometry, virtual-list/scroll state, zoom/current-page state, generation, and loading/error state; preserve `DocumentTabState` and `ImageTabState` internals.
- [ ] 3.3 Generalize common tab path/title/focus/safe-replacement/navigation/memory helpers and audit exhaustive matches so read-only-content checks include PDF while image-cache lifecycle checks remain image-specific.
- [ ] 3.4 Restrict dirty guards, quit checks, editing/IME/selection/formatting, search, save/autosave/recovery, outline, statistics, view-mode, export, external polling, and Markdown derivation to document tabs; add regression tests proving PDF activation cannot acquire or invalidate document state.
- [ ] 3.5 Extend the central supported-path router for PDF replace/new-tab behavior, duplicate-path focus, normal-form paths, recent files, workspace-root rebasing, and non-destructive open failure; ensure a dirty active document still follows the default open-target policy.
- [ ] 3.6 Include in-root PDF paths in generic per-workspace snapshots, restore, rename/move remapping, deletion handling, current-file marking, and missing-path pruning without adding a PDF-specific session schema field.
- [ ] 3.7 Add scope tests proving this change does not make CLI startup paths or OS external-drop opening accept PDFs and does not change image drop-to-import behavior.

## 4. Bounded Page Cache and Scheduling

- [ ] 4.1 Implement a root-crate `PdfPageCache` keyed by document identity, generation, page, and physical-width bucket, with pending/ready/error entries, claim counts, byte accounting, LRU order, and explicit GPUI image release.
- [ ] 4.2 Enforce the 32 MiB per-page and 64 MiB completed-raster budgets, no more than one claimed-page overshoot, immediate pay-down after release, and no persistent disk writes; add exact-boundary and overshoot/eviction tests.
- [ ] 4.3 Implement visible-range scheduling that requests only visible pages plus one neighbor on each side, prioritizes the current range, releases dormant/closed tab claims, and drops late results before GPUI conversion or cache admission.
- [ ] 4.4 Extend memory diagnostics to report PDF tab state, pending raw rasters, completed PDF page rasters, and combined image/PDF presentation memory without scanning document content or derived Markdown caches per frame.

## 5. PDF Page Surface and Controls

- [ ] 5.1 Build immutable page-geometry layout and a virtualized single-column PDF list with centered aspect-preserving placeholders, visible page boundaries, bounded rows per frame, stable reading-anchor current-page calculation, and no eager rasterization.
- [ ] 5.2 Connect ready page rasters to GPUI `RenderImage`, present bucketed rasters only at supported logical sizes, and render page-scoped loading/error placeholders while preserving scrolling and navigation.
- [ ] 5.3 Add the PDF-local toolbar with zoom out/in, 100% reset, fit width, 25–400% clamping, current/total page feedback, and validated page-number navigation; resize in fit-width mode shall invalidate only affected visible PDF raster keys.
- [ ] 5.4 Render document-level loading, corrupt/unsupported, encrypted, oversized, page-limit, missing-runtime, and vanished-file states as closable non-destructive PDF tabs without exposing raw native diagnostics.
- [ ] 5.5 Add GPUI tests for initial fit-width presentation, mixed-size page geometry, progressive page readiness, scrolling/current-page updates, page jumps, zoom boundaries, resize churn, stale render completion, page errors, and closing/replacing a PDF during pending work.

## 6. Workspace UI and Localization

- [ ] 6.1 Include PDF rows in background workspace scans, filename filters, hierarchy/sorting, selection, context actions, bounded rendering, and current-file styling; verify hidden/ignored directories and unsupported files remain excluded.
- [ ] 6.2 Disable or safely no-op document-only menus, shortcuts, sidebars, and status computations while a PDF is active, while preserving generic tab/file actions and restoring the prior document UI state after switching back.
- [ ] 6.3 Add PDF loading/control/navigation/error/action-unavailable message keys and complete translations in every supported language; extend localization completeness tests and ensure native error text never becomes user-facing copy.
- [ ] 6.4 Add application tests covering all interactive open entry points, mixed-case extensions, duplicate focus, current-tab preference behavior, PDF session restore, recent paths, rename/move/delete, dirty-document coexistence, close/quit, and document cache-identity preservation.

## 7. Native Packaging, Licensing, and Final Size Gates

- [ ] 7.1 Stage exactly one verified PDFium runtime into the installed resource location for each native target and update `packager.toml`/release CI without packaging the acquisition cache, archives, headers, import libraries, fixtures, debug files, other-target runtimes, V8/XFA, JavaScript, or fonts.
- [ ] 7.2 Update `THIRD_PARTY_NOTICES.md` and packaged notices for `pdfium-render`, PDFium, and required transitive notices; add dependency/license checks that reject an incomplete notice set or forbidden runtime feature.
- [ ] 7.3 Extend packaged-resource verification to inspect each NSIS, `.app`/DMG, DEB, and AppImage payload, assert exactly one matching runtime, and load/render a tiny PDF through the exact installed resource-discovery path with no network or external PDF tool.
- [ ] 7.4 Rerun the same-environment control/candidate size matrix after full integration, retain byte-exact reports for every format, and fail unless each compressed delta is at most 6 MiB and each installed/staged delta is at most 10 MiB.
- [ ] 7.5 Review final release dependency/features and largest-file reports; remove unused PDF features/assets and prove no persistent PDF raster cache or runtime download path contributes to installation or user-data storage.

## 8. Cross-Change Reconciliation and Verification

- [ ] 8.1 Re-read and reconcile overlapping path/tab/session requirements from `add-recent-workspace-switcher` and `open-documents-in-current-tab`, preserve the explicit non-goals owned by `add-drag-drop-open` and `support-cli-open-paths`, and validate every affected active change after any rebase.
- [ ] 8.2 Run formatting, `cargo test -p markion-pdf-viewer`, focused root PDF/file-tree/tab/cache/render tests, dependency-tree checks proving the member is GPUI-free, and `cargo test --workspace`; resolve failures without weakening cache or size limits.
- [ ] 8.3 Run `openspec validate add-pdf-document-viewing --strict`, the repository quality gate, and `openspec doctor`; keep all proposal/spec/design/task artifacts consistent with measured behavior and package evidence.
- [ ] 8.4 Manually smoke-test File → Open, Open in New Tab, Open Recent, file-tree/default-target gestures, continuous scrolling, zoom/page navigation, corrupt/encrypted/large files, close during render, offline runtime discovery, and memory stability on Windows x86_64, macOS arm64, Linux X11, and Linux Wayland.
