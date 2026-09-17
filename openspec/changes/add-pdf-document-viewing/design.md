## Context

The first implementation proved the native renderer, page UI, and package
budgets with PDFium built into the application. Product evaluation then found
that PDF reading is optional for many users. This design therefore keeps the
validated host presentation behavior while moving the native renderer across
the signed process/package seam defined by `add-first-party-plugin-platform`.

Accepted evidence remains useful: the built-in candidate added
2.97–3.77 MiB to installers and 8.03–8.55 MiB to installed payloads. The
final reusable plugin host adds only 274,133–339,968 compressed bytes and
367,422–794,099 installed bytes. Final native PDF plugins are
3,922,436–4,073,003 B compressed and 8,311,115–9,004,776 B installed.

## Goals / Non-Goals

**Goals:** preserve the complete read-only PDF experience; keep GPUI/editor
state in the host; make `.pdf` discoverable offline before installation;
require explicit installation; isolate PDFium and native handles in a verified
worker; keep scheduling, memory, and package sizes bounded; and preserve
document/cache identity invariants.

**Non-Goals:** a third-party marketplace, arbitrary UI injection, an in-process
Rust plugin ABI, an OS sandbox claim, PDF text/search/edit features, runtime
downloads, or broadening CLI/OS-drop entry points.

## Decisions

### 1. Deliver PDF through the first-party package platform

The signed bootstrap catalog declares `.pdf` → `dev.markion.pdf` →
`paged-document/v1` for every target and locale. The immutable registry combines
that declaration with core Markdown, text, and image handlers. Workspace scans
consume one snapshot; they perform no network/process work.

Known-but-uninstalled PDFs stay visible. Opening one creates a generic
plugin-document tab in a provider-missing state and opens the plugin surface.
Only an explicit, size-disclosing confirmation starts download/verification.
Install, enable, or rollback can reload affected tabs without restarting.

### 2. Keep native rendering exclusively in the official worker

`markion-pdf-viewer` remains GPUI-free and owns the pinned
`pdfium-render 0.9.3/pdfium_7881` binding. `markion-pdf-plugin` supplies the
framed worker entry point. It discovers `pdfium.dll`, `libpdfium.dylib`, or
`libpdfium.so` relative to its verified plugin directory and never searches
the system/current directory or downloads a runtime.

The root application does not depend on `markion-pdf-viewer` and core packages
do not stage PDFium or PDF-only notices. The built-in `markion-pdf` writer is
unrelated export functionality and remains in the core.

### 3. Use the typed `paged-document/v1` adapter

The host opens a supervised session for the exact signed active version. The
adapter exchanges only paths admitted by the host, document tokens, immutable
page geometry, bounded render requests, owned RGBA bodies, safe errors,
cancellation, and close messages. It applies source/page/body/frame/request
limits before allocation and rejects stale process or presentation generations.

The worker owns every PDFium handle. The root owns all GPUI elements and
`RenderImage` values; plugins cannot inject menus, windows, or editor state.

### 4. Generalize tab state, not the PDF UI copy

`WorkspaceTab::PluginDocument` contains provider ID, capability ID, normalized
path, request/document identity, immutable geometry, tracked scroll, zoom,
current page, generation, load state, and raster claims. It contains no text
model, dirty flag, undo, autosave, recovery, outline, statistics, Markdown
derivations, or export state. Host copy may continue to say “PDF” because the
official handler identity is PDF-specific; routing and state are generic.

Rename/move/delete, duplicate focus, recent files, workspace snapshots, and
restore operate on path-backed tabs. A current registry snapshot determines
whether a restored path is core content, a plugin document, conflicted, or no
longer supported.

### 5. Retain the bounded continuous page stream

Natural page geometry creates a virtual, centered, single-column layout and
total scroll height before raster completion. The host mounts visible rows and
one adjacent page on either side, prioritizes visible requests, and quantizes
physical widths into 64-pixel buckets. Fit-width resize and 25–400% numeric
zoom advance the tab generation so late results cannot enter the cache.

Each raster is at most 32 MiB. Completed rasters use a 64 MiB byte-aware LRU
with claims and explicit GPUI release. One claimed-page overshoot is allowed
and paid down on release. Rasters are never persisted. Page jumps and wheel or
keyboard scrolling update the same tracked scroll state and reading anchor.

### 6. Keep failure and lifecycle behavior contained

Missing, disabled, quarantined, incompatible, crashed, or uninstalled providers
produce localized, closable read-only states. Corrupt, encrypted, oversized,
too-many-page, missing-file, invalid-geometry, invalid-raster, and page failures
map to stable host categories; raw worker/native diagnostics remain bounded and
internal. Disabling/uninstalling stops the process, invalidates generations,
releases claims, and updates every affected tab.

### 7. Package and sign core and PDF independently

A `.markion-plugin` contains canonical `plugin.json`, its Minisign signature,
one target worker/runtime, and plugin notices. The manifest closes every member
by path, length, digest, and executable bit. A dedicated production plugin key
is distinct from the updater key; repository keys are test-only.

Native CI probes PDFium, runs fake/native fixture and framed-worker tests,
creates deterministic archives, inspects/extracts them defensively, emits exact
size/checksum/provenance/notice artifacts, and enforces 6 MiB / 10 MiB. It then
fills and signs the catalog, compiles core packages against that exact trust
root, and rejects any worker, archive, installed plugin tree, PDFium library,
or plugin-only notice in NSIS, DMG, DEB, or AppImage. The already accepted
same-runner host gate remains 1 MiB compressed / 2 MiB installed.

## Risks / Trade-offs

- Native workers are trusted first-party code, not an OS sandbox. The UI states
  this and version 1 accepts only Markion's signature.
- A process boundary adds one bounded raster copy. Raw bodies, width buckets,
  one renderer owner, deadlines, cancellation, and cache limits constrain it.
- Downloaded executables can encounter Gatekeeper, SmartScreen, or antivirus.
  Native launch-policy tests preserve OS metadata and fail closed; Markion does
  not strip quarantine or advise bypassing policy.
- Offline first use cannot render until the user has installed the plugin, but
  the signed bootstrap still explains the unavailable type and exact package
  size without a catalog fetch.

## Migration Plan

1. Keep the accepted built-in implementation and size reports as the behavior
   and payload control.
2. Land the signed catalog/store/supervisor/protocol platform and plugin UI.
3. Move hard-coded PDF routing/tab state to the immutable registry and generic
   plugin-document/paged-document host.
4. Build the official worker from the existing GPUI-free renderer and move
   PDFium plus notices into its archive.
5. Remove the root viewer dependency and packaged PDFium resources, then run
   host, protocol, UI, native, package-closure, and size gates.
6. Publish plugin assets and catalog independently. A rollback can disable
   plugin resolution without changing documents; incompatible payloads remain
   inert in per-user storage until removed by a compatible manager.
