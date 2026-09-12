## Context

See `proposal.md` — Why. Markion currently has an export-only `markion-pdf` workspace member that emits PDFs but does not parse them. Interactive opening routes supported paths through one application dispatcher, while `WorkspaceTab` distinguishes editable documents from read-only images. Image presentation already demonstrates the required ownership split: background decoding produces pure pixels, the root crate adapts them to GPUI `RenderImage`, and a byte-bounded LRU owns completed GPU-facing rasters.

PDF viewing adds two constraints that the single-image path does not have. A document can contain thousands of differently sized pages, so the application needs virtual page geometry before rasters exist and must not decode all pages eagerly. PDFium also makes no thread-safety guarantee, so native calls need one serialized owner even though requests and result delivery remain asynchronous.

The size feasibility study used Markion v0.3.6 release artifacts as a reference: Windows NSIS 20.32 MiB, macOS arm64 DMG 25.35 MiB, Linux DEB 29.76 MiB, and Linux AppImage 28.51 MiB; the extracted Linux payload was 63.17 MiB. Matching non-V8 PDFium 7881 archives are approximately 3.4–3.6 MiB per target, while the runtime library is approximately 7–8 MiB unpacked. These figures justify the 6 MiB packaged / 10 MiB installed delta budgets but are not acceptance evidence; implementation must produce same-environment control and candidate measurements.

Several active changes touch adjacent wording or code: `add-recent-workspace-switcher`, `open-documents-in-current-tab`, `add-drag-drop-open`, and `support-cli-open-paths`. The first two should include PDF paths through their generic path/tab seams after rebasing. This change deliberately does not add PDF handling to the latter two entry points.

## Goals / Non-Goals

**Goals:**

- Keep PDF parsing and rasterization behind a small GPUI-free interface, with exactly one owner of native handles and no PDF work in frame rendering.
- Make page geometry, visible-range scheduling, cache identity, cancellation, and byte accounting explicit and deterministic.
- Ship an offline renderer on every supported release target without bundling a browser engine, fonts, development files, or another platform's runtime.
- Turn package and installed-size limits into failing, byte-exact acceptance gates rather than advisory release notes.
- Reuse content-independent tab/path behavior while making document-only state access type-checked at the application boundary.

**Non-Goals:**

- Reusing or extending the export writer as a parser; PDF writing and viewing remain separate modules.
- Building a selectable text layer, search index, link/outline UI, password workflow, annotations, forms, printing, editing, or JavaScript execution.
- Sandboxing the renderer in a separate process in this first version; the renderer is serialized on a dedicated thread, with process isolation retained as a future hardening option.
- Making a network download part of normal application startup. Developer/runtime acquisition is an explicit build step, while release packages are self-contained.
- Persisting transient page position, zoom, native document handles, or page rasters in `session.toml` or on disk.

## Decisions

### 1. Use a dynamically loaded, non-V8 PDFium runtime through a narrow Rust binding

Create `crates/pdf-viewer` with package name `markion-pdf-viewer`. It will use a pinned `pdfium-render` version and the matching PDFium API/build, with default features disabled and only the binding features required for raw bitmap rendering. It will not enable V8, XFA, static linking, PDF editing APIs, or broad image-format decoding. The crate returns pure metadata, typed errors, and owned RGBA page rasters; it contains no `gpui` dependency.

The release package carries PDFium as a dynamically loaded sidecar. Dynamic loading keeps the large native payload separately measurable, makes a bad runtime discoverable as a typed initialization failure, and avoids a platform-specific static C++ link configuration. PDFium's own license and all required third-party notices are added to `THIRD_PARTY_NOTICES.md` and the packaged notices.

Alternatives considered:

- PDF.js already provides viewer UI but requires introducing a WebView and substantially complicates Linux/AppImage runtime dependencies; it conflicts with Markion's native no-WebView architecture.
- Hayro avoids a native sidecar but currently describes itself as experimental/work-in-progress and does not yet provide the compatibility/performance confidence required for arbitrary user PDFs.
- MuPDF is technically mature but requires AGPL compliance or a commercial license, which is unsuitable for the current MIT distribution without a separate product decision.
- Windows/macOS platform APIs plus Poppler on Linux minimize some platform payloads but create three behavior and QA implementations and an inconsistent Linux dependency.

References: `https://docs.rs/pdfium-render/0.9.3/pdfium_render/`, `https://github.com/bblanchon/pdfium-binaries/releases/tag/chromium%2F7881`, and `https://pdfium.googlesource.com/pdfium/+/HEAD/LICENSE`.

### 2. Give one dedicated service thread exclusive ownership of PDFium

The GPUI-free member exposes a service interface conceptually shaped as:

```text
PdfCommand
├─ Open { request_id, path }
├─ Render { document_id, generation, page, target_width_px }
├─ Close { document_id }
└─ Shutdown

PdfEvent
├─ Opened { request_id, document_id, page_sizes }
├─ PageReady { document_id, generation, page, raster }
└─ Failed { scope, safe_error_kind }
```

A bounded command queue feeds one long-lived worker thread. That thread initializes the dynamic library once, owns every native document/page handle, and executes exactly one native call at a time. Application code never retains a PDFium pointer or invokes a binding directly. Results carry only owned Rust data and are delivered back through GPUI tasks.

Render commands carry a monotonically increasing tab generation. Closing/replacing a tab, reopening a changed path, or changing zoom increments the generation; result admission checks `(document_id, generation)` before constructing or claiming a GPUI image. The queue coalesces superseded render requests and prioritizes the current visible range, so rapid scrolling cannot build an unbounded FIFO backlog.

Using `pdfium-render`'s global mutex from arbitrary background tasks was rejected because it hides queue growth and priority. A dedicated owner makes serialization, shutdown, stale-result rejection, and memory accounting observable. A separate renderer process would contain a native crash more strongly, but IPC, lifecycle recovery, and duplicate-binary/package concerns make it disproportionate for this preview-only change.

### 3. Add a third workspace-tab variant without weakening the document model

Extend the existing sum type rather than adding PDF option fields to document or image state:

```text
WorkspaceTab
├─ Document(DocumentTabState)  canonical editable Markdown/text state
├─ Image(ImageTabState)        one read-only decoded-image identity
└─ Pdf(PdfTabState)            document token, page geometry, view state
```

`PdfTabState` contains a normalized path, open/request identity, optional immutable page geometry, virtual-list/scroll state, zoom mode, current page, generation, and loading/error state. It contains no PDF bytes, native handles, Markdown document, undo, dirty, autosave, recovery, outline, statistics, or export fields.

Common helpers expose `path`, `title`, content kind, safe-to-replace state, focus identity, and transient presentation memory. Document-only helpers continue to return checked document references. Existing `is_image()` conditions that really mean "read-only content" are generalized deliberately; conditions that truly own image-cache claims remain image-specific. Compiler exhaustiveness plus focused tab/action tests are used to audit every match.

### 4. Use one supported-path classifier and preserve entry-point boundaries

Replace binary document/image classification with a shared `SupportedPathKind::{Document, Image, Pdf}` used by interactive opening, the file tree, icons, recent files, session restoration, and path remapping. File → Open and file-tree gestures keep the active `open-documents-in-current-tab` target policy; duplicate paths focus their existing tab before any second native document opens.

PDF paths participate in per-workspace snapshots through the generic path-backed tab representation introduced by `add-recent-workspace-switcher`; no schema field specific to PDF is added. Older builds encountering a recorded PDF path skip it as unsupported, so session compatibility is one-way additive and needs no migration.

OS-originated image drop currently means import-as-resource, while the separate drag/drop change owns document opening. CLI startup support is also a separate active change. Neither classifier call site is broadened to PDF here; tests pin that scope so a shared enum does not accidentally expand those behaviors.

### 5. Separate virtual page geometry from raster ownership

Opening a document first obtains page count and natural page sizes. The root PDF surface builds a virtual single-column layout from those immutable dimensions, zoom mode, viewport width, and fixed page gaps. It precomputes only compact page placements and a total scroll height, then mounts page nodes for the current visible range while a plain tracked scroll container supplies continuous pixel scrolling. Placeholder height is therefore known before rendering, so scrolling and page navigation never depend on decoded raster availability or GPUI measuring an off-screen variable-height row.

```text
path
  → serialized Open
  → Arc<[PageGeometry]>
  → virtual visible range + one neighbor on each side
  → prioritized Render requests (document, generation, page, width bucket)
  → owned RGBA raster
  → root-crate GPUI RenderImage
  → PdfPageCache claim / paint / release
```

Layout uses exact logical zoom, while render identity quantizes the requested physical width upward into a small fixed bucket (initially 64 physical pixels). A cached raster can be presented smaller within its bucket but never enlarged beyond its pixel coverage. This prevents continuous resize from producing a new raster for every pixel while preserving crispness. Fit-width resize invalidates only affected visible PDF keys; numeric zoom actions use discrete values inside the specified 25–400% range.

The current page is the page intersecting a stable reading anchor near the top of the viewport. Page-number navigation maps the requested page through the precomputed placement table and sets the tracked scroll offset before requesting its raster. The PDF toolbar is part of the PDF surface, not the global Markdown view-mode controls.

### 6. Add a dedicated bounded PDF page cache with explicit GPU release

The root crate owns `PdfPageCache`, keyed by normalized document identity, generation, page index, and physical-width bucket. Each ready entry stores its `Arc<RenderImage>`, exact byte length, pixel dimensions, and logical presentation dimensions. Pending and typed-error entries are bounded by count; ready entries use byte-aware LRU eviction and claim counts.

Hard limits mirror the spec:

- at most 32 MiB retained bytes for one page raster;
- 64 MiB total completed raster budget;
- only visible pages plus one adjacent page on each side may be pending/claimed for preloading;
- at most one page of temporary claimed overshoot, paid down immediately after release;
- stale generation/zoom results are dropped before cache admission;
- `cx.drop_image` is called for evicted/replaced GPUI images;
- no disk cache.

Source size (512 MiB) is checked before open, and page count (10,000) before page geometry is accepted. Target pixel dimensions are reduced before native allocation to satisfy the per-page byte ceiling. These are independent of the existing Markdown preview-image cache so the two ownership systems cannot evict each other's live keys, but memory diagnostics report both and their combined total.

### 7. Stage one checksum-pinned runtime explicitly; builds never download implicitly

Add a small repository-controlled runtime manifest containing the PDFium build/API version, target triple, source URL, SHA-256 digest, expected archive member, and expected uncompressed byte range. A PowerShell 7 script downloads and verifies only the current target into an ignored `target/pdfium-runtime/<triple>/` directory, extracts only the runtime library, and rejects path traversal, unexpected members, digest mismatch, debug artifacts, or a runtime outside the expected size range.

`build.rs` and ordinary Cargo dependency resolution do not access the network. Local developers run the staging script explicitly when exercising real PDF rendering; pure state/cache tests use a fake renderer. Native CI runs the script before renderer integration tests and packaging. `packager.toml` stages the one current-target library into a stable application resource location. Release runtime discovery resolves only that packaged location. Non-release discovery prefers an explicit developer override, then a packaged resource, and may finally use the matching target runtime in the repository-local staging directory created by the checksum-validating script. Neither mode searches arbitrary current-working-directory or system libraries.

Packaged verification checks the actual NSIS, `.app`/DMG, DEB, and AppImage payload path, not merely `target/release`, and opens/renders a tiny non-encrypted fixture through that path. Headers, import libraries, archives, fixtures, and the acquisition cache are excluded from packages.

### 8. Make size acceptance reproducible and non-negotiable

The first implementation step is a packaging spike, before application integration. On each native runner it builds/packages a PDF-disabled control from the recorded pre-change revision and a minimal candidate using the same Rust toolchain, root lockfile policy, release profile, assets, and cargo-packager version. A checked-in budget manifest records the immutable base revision and byte ceilings, not mutable "expected" numbers.

The size verifier writes a machine-readable report containing target, format, base revision, tool versions, control bytes, candidate bytes, delta bytes, staged/installed file totals, and the largest added/changed files. It applies:

```text
compressed NSIS / DMG / DEB / AppImage delta  <= 6 * 1,048,576 bytes
installed or format-native staged delta       <= 10 * 1,048,576 bytes
```

Package size is the final artifact file length. Installed size uses the format-native application tree when available (`.app`, extracted DEB/AppImage); the Windows job uses the exact staged NSIS application payload manifest because unattended installation would mutate runner state. Both control and candidate use the same measurement rule. Reports are uploaded with native CI artifacts and summarized in job output. Any missing control, unmeasurable payload, or exceeded limit is a failure. The spike stops the change before broad integration if the minimal runtime cannot fit; budgets are not relaxed inside this change.

This same gate runs again after the complete feature so Rust glue, localization, fixtures accidentally entering resources, and packaging metadata are included. A future feature that needs more space must propose an explicit budget change rather than editing the manifest opportunistically.

### 9. Treat native-parser maintenance as part of the feature

The build disables PDF JavaScript and XFA, validates runtime provenance, bounds source/page/raster work, and maps binding errors to a small safe error enum. Native error strings may be logged diagnostically but are not interpolated into localized UI. Password-protected files are detected and rejected without accepting, storing, or logging a password.

The pinned runtime is reviewed during dependency updates and security advisories. Upgrading PDFium requires updating the binding/API pin, all target digests and expected sizes together, rerunning renderer fixtures and native packaging verification, and proving the size budgets again. Silent system-library fallback is prohibited in releases because it makes behavior and security patch level unknowable.

## Risks / Trade-offs

- **[Native PDFium defect can still crash or compromise the Markion process]** → Disable active-content features, pin and verify the runtime, maintain prompt security updates, fuzz/fixture-test the wrapper boundary, and revisit same-executable helper-process isolation if crashes appear in real or fuzzed inputs.
- **[A serialized renderer can lag during rapid scrolling]** → Bound/coalesce the queue, prioritize the latest visible range, use width buckets, drop stale results, and never let rendering block input or frame construction.
- **[Large/irregular pages can exhaust CPU or GPU memory]** → Enforce source/page/per-raster/cache limits before allocation, virtualize geometry, preload only one neighbor, and explicitly release GPUI images.
- **[The native runtime is absent or staged in the wrong package location]** → Resolve a single deterministic release path and make each format's packaged smoke test mandatory.
- **[The feature exceeds its storage budget after Rust glue and localization are linked]** → Run the size spike first and the identical gate after full integration; exclude optional binding features and all non-runtime archive members; stop rather than relaxing the budget.
- **[PDFs render differently across operating systems because of font substitution or color handling]** → Use PDFs with embedded fonts in deterministic pixel tests, add native smoke fixtures for system-font fallback, and treat minor antialiasing variation as non-golden manual evidence.
- **[Overlapping active changes produce conflicting tab/session wording or routing]** → Re-read and rebase their delta specs before archive, keep PDF scope in additive requirements where possible, and run strict validation for every active change.
- **[The repository gains a second similarly named PDF crate]** → Keep `markion-pdf` documented as export/write and `markion-pdf-viewer` as parse/render; neither depends on the other or on GPUI.

## Migration Plan

1. Record the exact pre-change base revision and complete the three-platform runtime/package-size spike. Stop and revise the proposal if any minimal candidate breaches a hard budget.
2. Add the GPUI-free renderer member, fake backend, limits, fixtures, explicit runtime acquisition, and headless tests without changing supported application paths.
3. Add `Pdf` tab state, classification, open/session/file-tree integration, virtualized surface, cache, controls, and localization behind one compile-time integration seam until application tests pass.
4. Enable the bundled runtime in each native package, run packaged smoke tests, produce final size reports, and verify licenses and excluded archive members.
5. Rebase overlapping active OpenSpec deltas, run the repository quality gate, `cargo test --workspace`, strict OpenSpec validation, and manual native PDF checks before archive.

No persisted file or preference migration is required. Session files remain compatible because they store generic paths; older versions skip `.pdf` entries they do not support. Rollback removes the PDF classifier/tab/runtime and leaves source PDFs untouched. A release rollback may ship the previous application without rewriting user documents or configuration.
