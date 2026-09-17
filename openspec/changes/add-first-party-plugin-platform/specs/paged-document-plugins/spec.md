## Purpose

Defines host-rendered, read-only paged documents whose metadata and page pixels are supplied by an optional supervised plugin, with PDF as the first official provider.

## ADDED Requirements

### Requirement: Known plugin document types SHALL remain discoverable without bundled providers
Markion SHALL recognize file types advertised by its signed official catalog even when the corresponding plugin is not installed or enabled. Such files SHALL remain eligible for the Files panel and interactive open flows, but opening one SHALL show a non-destructive localized surface that identifies the required plugin and offers an explicit install or enable action. Merely scanning or opening the file SHALL NOT begin a download.

#### Scenario: PDF plugin is absent
- **WHEN** the user opens a local `.pdf` file and the official PDF plugin is not installed
- **THEN** Markion keeps the file untouched and shows that PDF viewing requires the official PDF plugin
- **AND** no package download begins until the user confirms installation

#### Scenario: Provider is installed but disabled
- **WHEN** the user opens a known plugin document type whose compatible provider is installed but disabled
- **THEN** Markion offers an explicit Enable action rather than reporting the file as corrupt or unsupported
- **AND** declining preserves the current plugin and document state

### Requirement: Paged document presentation SHALL remain host-owned
For an active paged-document provider, Markion SHALL own the read-only tab, continuous single-column page stream, fit-width and numeric zoom, page-number navigation, loading/error placeholders, current-page feedback, raster cache, GPU image lifecycle, and all localized chrome. The provider SHALL supply only typed document metadata and owned page rasters through its capability interface. Provider work SHALL execute outside GPUI frame rendering and SHALL NOT receive or construct GPUI values.

#### Scenario: Provider opens a multi-page document
- **WHEN** a compatible plugin returns valid page geometry for an opened file
- **THEN** the host presents ordered virtual page placeholders and progressively requests visible pages plus its bounded neighborhood
- **AND** no provider parse or raster operation runs synchronously in a frame

#### Scenario: User navigates and zooms
- **WHEN** the user scrolls, enters a page number, selects fit width, or changes numeric zoom
- **THEN** the host updates geometry, current-page state, cache identity, and bounded render requests
- **AND** the provider receives only the typed request needed to render the applicable page bucket

### Requirement: The official PDF provider SHALL preserve bounded viewing behavior
The official PDF plugin SHALL preserve the `add-pdf-document-viewing` behavior for case-insensitive local `.pdf` paths, serialized native calls, corrupt/encrypted/oversized/page-limit errors, page-scoped failures, runtime provenance, and offline rendering after installation. It SHALL retain the 512 MiB source limit, 10,000-page limit, 32 MiB completed-page limit, 64 MiB host PDF-raster budget, visible range plus one neighbor scheduling, and generation-based stale-result rejection. It SHALL include exactly one verified non-V8/non-XFA PDFium runtime for its own target and SHALL NOT download a runtime during use.

#### Scenario: Installed PDF plugin opens offline
- **WHEN** the compatible PDF plugin is installed and the user opens a supported PDF without network access
- **THEN** the plugin loads its packaged target PDFium runtime and supplies page geometry and rasters through the paged-document capability
- **AND** it performs no runtime or font download

#### Scenario: PDF exceeds a safety limit
- **WHEN** a PDF exceeds the source, page-count, or per-page raster limit
- **THEN** the provider returns the corresponding stable safe category before unsafe work or allocation
- **AND** the host shows a localized contained state and leaves all source files and documents unchanged

#### Scenario: Rapid PDF scrolling supersedes work
- **WHEN** visible PDF pages change faster than the plugin can render them
- **THEN** the host and provider prioritize the newest visible range, coalesce or cancel superseded requests, and reject stale generation results
- **AND** completed host raster memory remains within the existing bounded overshoot rules

### Requirement: Plugin document tabs SHALL preserve generic path and document isolation behavior
A plugin-provided document tab SHALL participate in path de-duplication, title, recent files, workspace snapshots, current-file marking, rename/move remapping, deletion handling, navigation, close, and safe replacement under the same generic path-backed rules as other read-only content. It SHALL carry no editable Markdown state, dirty flag, autosave, recovery, undo, outline, statistics, export state, or document-derived caches.

#### Scenario: Existing plugin document is reopened
- **WHEN** the user opens a path that already has an active plugin-document tab
- **THEN** Markion focuses the existing tab without starting a second provider document
- **AND** preserves its transient position, zoom, and ready host rasters

#### Scenario: Switching through a plugin document preserves an editor tab
- **WHEN** the user switches from a dirty Markdown document to a plugin-provided PDF and back
- **THEN** the Markdown text, selection, undo history, dirty state, scroll state, and derived cache identities remain unchanged
- **AND** the plugin tab never acquires editable-document state

#### Scenario: Provider becomes unavailable while a tab is open
- **WHEN** the plugin is disabled, uninstalled, crashes, or becomes incompatible while its tab exists
- **THEN** the tab transitions to a closable localized provider-unavailable state and releases process/raster claims
- **AND** no late plugin result can populate another tab

