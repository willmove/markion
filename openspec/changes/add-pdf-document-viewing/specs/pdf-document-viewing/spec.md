## Purpose

Defines how Markion opens local PDF references as bounded, read-only, multi-page content while preserving responsive native rendering and the editing state of Markdown documents.

## ADDED Requirements

### Requirement: Supported local PDFs SHALL open as read-only content
Markion SHALL recognize an existing local file with the case-insensitive `.pdf` extension as supported PDF content. File → Open, Open in New Tab, Open Recent, and file-tree opening SHALL route that path to a read-only PDF tab rather than decoding it as UTF-8 text. A successfully routed PDF path SHALL participate in normal tab title, path de-duplication, current-file marking, recent-file, workspace-root, path-backed session snapshot, navigation, rename/remap, delete, and close behavior. Opening PDF content SHALL NOT make CLI startup paths or OS drag/drop accept PDFs as part of this change.

#### Scenario: File Open targets a PDF tab
- **WHEN** the user selects an existing `.pdf` file through File → Open and the default open-target rule permits replacing the active tab
- **THEN** the active tab is replaced by a read-only PDF tab for that path
- **AND** the PDF bytes are not interpreted as document text

#### Scenario: New-tab flows target a PDF tab
- **WHEN** the user opens a PDF through Open in New Tab, Open Recent, or a new-tab file-tree gesture
- **THEN** Markion appends and activates a read-only PDF tab, or focuses the existing tab for that path

#### Scenario: Extension matching is case-insensitive
- **WHEN** a supported path uses an uppercase or mixed-case extension such as `.PDF` or `.Pdf`
- **THEN** Markion routes it through the same PDF-viewing path

#### Scenario: Existing PDF tab is reused
- **WHEN** the user opens a PDF path that is already open in the current window
- **THEN** Markion focuses the existing PDF tab without loading a second document or appending a duplicate tab
- **AND** that tab preserves its page position, zoom mode, and cached visible page state

#### Scenario: PDF tab restores with its workspace
- **WHEN** a path-backed workspace snapshot contains an in-root PDF path that still exists
- **THEN** restoring that workspace reopens the path as a PDF tab in its recorded order
- **AND** the restored PDF starts with fresh transient page position and raster-cache state

### Requirement: PDF tabs SHALL present a virtualized continuous page stream
A ready PDF tab SHALL display pages in document order as a centered, single-column, vertically scrollable stream with visible page boundaries and preserved page aspect ratios. The initial presentation SHALL use fit-width mode without enlarging a page beyond the configured maximum raster limits. The PDF toolbar SHALL expose zoom out, zoom in, reset to 100%, fit width, a current-page/total-page indicator, and page-number navigation. Numeric zoom SHALL remain between 25% and 400% inclusive. Resizing the content area while fit-width mode is active SHALL recompute presentation geometry without eagerly rendering every page.

#### Scenario: PDF opens in fit-width mode
- **WHEN** document metadata loads successfully for a multi-page PDF
- **THEN** the first page becomes visible in fit-width mode and later pages occupy ordered virtual placeholders below it
- **AND** pages render progressively without waiting for the entire document

#### Scenario: Scrolling advances the current page
- **WHEN** the user scrolls through the continuous page stream
- **THEN** the current-page indicator tracks the page nearest the reading position
- **AND** pages outside the bounded render neighborhood remain placeholders or are evicted rasters

#### Scenario: Page navigation reveals a requested page
- **WHEN** the user enters a valid page number between one and the document page count
- **THEN** the stream scrolls to reveal that page and schedules it for rendering
- **AND** an invalid or out-of-range value is rejected or clamped without leaving the document in an unusable position

#### Scenario: Zoom stays within bounds
- **WHEN** the user invokes zoom controls at or beyond 25% or 400%
- **THEN** the effective numeric zoom remains within that inclusive range
- **AND** changing zoom invalidates only PDF presentation rasters for the affected tab, not Markdown-derived state

#### Scenario: Fit width follows the viewport
- **WHEN** the user resizes the window while fit-width mode is active
- **THEN** visible page presentation widths follow the available PDF surface width while preserving aspect ratio
- **AND** the application does not synchronously rasterize off-screen pages during the resize

### Requirement: PDF loading and rendering SHALL be finite and off the render path
PDF file inspection, document loading, page metadata access, and page rasterization SHALL execute outside GPUI frame rendering. Native PDF calls SHALL be serialized so no two calls execute concurrently in the process. Markion SHALL schedule only visible pages plus at most one adjacent page before and after the visible range. Each completed page raster SHALL be limited to 32 MiB, and the completed PDF raster cache SHALL be limited to 64 MiB; when an immediately visible raster temporarily requires the cache to exceed that total, the overshoot SHALL be limited to one page and SHALL be paid down as soon as the page is no longer claimed. Source files larger than 512 MiB and documents reporting more than 10,000 pages SHALL be rejected before page rendering. PDF rasters SHALL remain memory-only and SHALL NOT be written to a persistent disk cache.

Release builds SHALL discover PDFium only at the packaged target-specific resource path. Development builds SHALL prefer an explicit developer override, then a packaged resource, and MAY finally use the target-specific runtime produced beneath the repository `target/` directory by the checksum-validating staging script. Neither build mode SHALL search the current working directory or system library paths, and application startup SHALL NOT download a runtime.

#### Scenario: Frame rendering encounters an uncached page
- **WHEN** a visible page has no ready raster at the current zoom bucket
- **THEN** the frame renders a bounded placeholder and enqueues background work
- **AND** no native PDF parse or raster call runs synchronously in that frame

#### Scenario: Rapid scrolling bounds scheduled work
- **WHEN** the visible range changes repeatedly before pending page renders complete
- **THEN** newly relevant visible and adjacent pages take priority
- **AND** stale results outside the current document generation or zoom bucket are discarded without entering the cache

#### Scenario: Cache reaches its byte limit
- **WHEN** admitting a completed page would exceed the 64 MiB completed-raster budget
- **THEN** least-recently-used unclaimed rasters are evicted until the cache is within budget
- **AND** a currently visible raster can overshoot the budget by no more than its 32 MiB per-page limit before later release pays down the overshoot

#### Scenario: Source safety limit is exceeded
- **WHEN** a PDF exceeds the 512 MiB source-size limit or reports more than 10,000 pages
- **THEN** Markion shows a localized unavailable-PDF state before rendering any page
- **AND** no existing tab or file is modified

#### Scenario: A development build uses an explicitly staged runtime
- **WHEN** Markion runs as a development build without a packaged PDFium resource or developer override and the repository staging script has produced the matching target runtime
- **THEN** runtime discovery loads that target-scoped staged file and PDF viewing works under a normal `cargo run`
- **AND** release discovery remains restricted to the packaged resource path

#### Scenario: Closing a PDF releases transient resources
- **WHEN** a PDF tab closes or is replaced while metadata or raster work is pending
- **THEN** its document generation becomes stale, completed late results are discarded, and all unshared raster claims are released
- **AND** no persistent raster files remain on disk

### Requirement: PDF tabs SHALL remain outside editable-document state
A PDF tab SHALL carry no Markdown text model, cursor/selection, IME composition, undo/redo history, dirty flag, autosave timer, recovery snapshot, outline, document statistics, view mode, formatting state, or export state. Document-only commands and affordances SHALL be disabled or shall leave all state unchanged while a PDF is active. Switching through a PDF tab SHALL preserve every other document tab's content, selection, history, scroll positions, dirty state, and cached derived state.

#### Scenario: Document command is invoked on a PDF
- **WHEN** the user invokes editing, formatting, save, autosave, recovery, outline, statistics, view-mode, or export behavior while a PDF tab is active
- **THEN** the PDF file and every open document remain unchanged
- **AND** Markion does not create dirty, undo, autosave, recovery, or Markdown-derived state for the PDF

#### Scenario: Switching through a PDF preserves a document
- **WHEN** the user switches from an editable document to a PDF and later switches back
- **THEN** the document's text, selection, history, scroll positions, dirty state, and cached per-version derived state retain their prior identities and values
- **AND** the PDF is never parsed as Markdown

#### Scenario: Closing a PDF needs no dirty confirmation
- **WHEN** the user closes a PDF tab or quits with PDF tabs and no dirty document tabs open
- **THEN** no unsaved-document confirmation is shown because of the PDFs

### Requirement: PDF failures SHALL remain contained and recoverable
If a supported PDF cannot be read, has invalid structure, is encrypted or password-protected, loses its backing file, or fails metadata or page rendering, its tab SHALL show a localized unavailable-PDF state identifying the affected path and a safe failure category. Raw native-library diagnostics SHALL NOT be exposed as user-facing text. A page-specific failure SHALL remain scoped to that page when the rest of the document is usable. The failure SHALL NOT mutate the source file, replace another tab, invalidate document caches, or prevent the tab from closing.

#### Scenario: Corrupt PDF fails during load
- **WHEN** a `.pdf` path exists but cannot be parsed as a valid PDF
- **THEN** its tab shows a localized corrupt-or-unsupported PDF state
- **AND** the tab remains closable and other content remains usable

#### Scenario: Encrypted PDF is unsupported by the MVP
- **WHEN** the selected PDF requires a password
- **THEN** its tab shows a localized encrypted-PDF-unavailable state
- **AND** Markion does not prompt for, store, log, or transmit a password

#### Scenario: One page fails to render
- **WHEN** document metadata is usable but a particular page raster fails
- **THEN** that page shows a localized page-unavailable placeholder
- **AND** navigation to other pages remains available

#### Scenario: Backing file disappears
- **WHEN** a PDF becomes unreadable or disappears before pending work completes
- **THEN** late work cannot replace another document generation and the PDF tab transitions to a contained unavailable state
