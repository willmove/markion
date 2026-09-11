## Why

PDFs commonly sit beside Markdown notes as references, but Markion hides them from the workspace and cannot inspect them without switching applications. A bounded native preview would make those references available in the existing tab workflow, provided the feature does not compromise Markion's lightweight distribution or cached rendering architecture.

## What Changes

- Recognize local `.pdf` files case-insensitively and open them from File → Open, Open in New Tab, Open Recent, and the Files sidebar as read-only PDF tabs with normal path de-duplication, workspace, recent-file, and session behavior.
- Present PDFs as a virtualized vertical page stream with fit-width as the default, bounded zoom controls, page position feedback/navigation, localized loading and contained error states, and no editable-document behavior.
- Render only visible and nearby pages off the GPUI render path through a serialized, GPUI-free PDF renderer service, then retain page rasters in a byte-bounded LRU cache. PDF work must not invalidate or recompute any document-version Markdown cache.
- Bundle exactly one pinned, non-V8/non-XFA PDFium runtime for the release target and add reproducible package-size reporting and hard acceptance budgets: no packaged artifact may grow by more than 6 MiB and no installed/staged payload may grow by more than 10 MiB versus a same-toolchain pre-change control build (1 MiB = 1,048,576 bytes).
- Fail CI and block completion of the change when either size budget is exceeded; record per-platform baseline, candidate, and delta bytes as release evidence rather than silently relaxing the limit.

Non-goals: text selection or copy, full-text search, outlines/bookmarks, annotations, form filling, PDF JavaScript, XFA, printing, PDF editing, embedded WebView/PDF.js, persistent page-raster disk caches, bundling fonts, file-association registration, or expanding the separate CLI and OS drag/drop changes.

## Capabilities

### New Capabilities

- `pdf-document-viewing`: Local PDF classification, read-only tab behavior, virtualized page presentation, zoom/page navigation, bounded rendering and cache lifecycle, and contained failures.

### Modified Capabilities

- `workspace`: Include PDF files in bounded workspace discovery, filtering, opening, current-file marking, and path-backed workspace snapshots.
- `markdown-editing`: Generalize the heterogeneous tab host and document-only guards so PDF tabs coexist with document and image tabs without disturbing isolated editing state or derived Markdown caches.
- `release-packaging`: Bundle one target-specific PDFium runtime and make compressed-package and installed-payload size budgets measurable release gates.
- `ui-i18n`: Localize PDF loading, navigation, zoom, unavailable/encrypted-file, and action-unavailable strings in every supported language.

## Impact

- Adds a GPUI-free workspace member (separate from the existing export-only `markion-pdf` writer) that owns PDFium loading, document/page metadata, serialized render requests, limits, and pure raster results; GPUI image conversion and presentation remain in the root crate.
- Extends `WorkspaceTab`, open-path routing, file-tree classification/icons, recent/session restoration, document-only command guards, memory accounting, and root workspace rendering.
- Adds a pinned `pdfium-render` binding with a target-specific dynamically loaded PDFium runtime, checksum-pinned acquisition/staging, third-party notices, and packaged-runtime verification on Windows x86_64, macOS arm64, Linux DEB, and Linux AppImage.
- Adds PDF page virtualization and a dedicated byte-bounded raster cache. It preserves bounded file-tree rendering, shared per-version Markdown derivations, syntax-highlight memoization, cached text handles, and GUI-free member-crate boundaries.
- Coordinates with the active `add-recent-workspace-switcher`, `open-documents-in-current-tab`, `add-drag-drop-open`, and `support-cli-open-paths` changes before archive so overlapping path/tab wording is rebased without silently broadening this change's scope.
