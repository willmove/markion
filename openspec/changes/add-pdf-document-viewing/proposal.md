## Why

PDFs commonly sit beside Markdown notes as references, but not every Markion
user needs a 7–9 MiB native PDF runtime. PDF reading should participate in the
normal workspace and tab flow without forcing that payload into every core
installation. The active `add-first-party-plugin-platform` change supplies the
signed, explicitly installed provider boundary this feature now targets.

## What Changes

- Recognize `.pdf` through the signed file-handler registry and keep the type
  visible even when its official provider is not installed.
- Opening an uninstalled/disabled/incompatible provider creates a closable
  read-only provider-unavailable tab and exposes Preferences → Plugins; it
  never downloads or installs code silently.
- After explicit installation, route PDF paths to a generic plugin-document
  tab backed by the official `dev.markion.pdf` process and
  `paged-document/v1` capability.
- Preserve the host-owned continuous page stream, fit-width and numeric zoom,
  page navigation, localized safe failures, visible-page scheduling, 32 MiB
  raster limit, and 64 MiB cache.
- Build PDFium, its worker, and PDF-only notices only into independently signed
  `.markion-plugin` archives. Core NSIS, DMG, DEB, and AppImage packages carry
  only the small signed catalog/host and must contain no PDF worker or runtime.
- Enforce separate budgets: the reusable core host stays within 1 MiB
  compressed / 2 MiB installed over its control, while each PDF plugin stays
  within 6 MiB compressed / 10 MiB installed.

Non-goals remain text selection/copy, PDF search, outlines, annotations,
forms, JavaScript/XFA, printing, editing, OCR, persistent raster caches, CLI or
OS-drop expansion, and a third-party plugin marketplace.

## Capabilities

### New Capabilities

- `pdf-document-viewing`: Optional official PDF provider behavior plus the
  host-owned read-only multi-page experience.

### Modified Capabilities

- `workspace`: Resolve PDF visibility, opening, recent files, and restoration
  through the immutable generic handler registry.
- `markdown-editing`: Keep generic plugin-document tabs outside every editable
  document state and derived cache.
- `release-packaging`: Publish PDF as a separately signed, size-gated plugin
  while proving all core packages are closed over optional payloads.
- `ui-i18n`: Localize PDF presentation/failures and the missing-provider path;
  signed plugin identity is complete in every supported locale.

## Impact

- The root package no longer depends on `markion-pdf-viewer`; the existing
  export-only `markion-pdf` dependency remains because PDF export is a separate
  built-in feature.
- `markion-pdf-viewer` remains GPUI-free and is consumed by the official worker
  crate. PDFium discovery and native handles exist only in that process.
- `WorkspaceTab::PluginDocument` and the generic paged-document host own UI,
  path identity, geometry, scheduling, and raster cache state.
- Release CI builds, signs, inspects, tests, measures, publishes, and mirrors
  the PDF archives/catalog independently, then compiles the core against that
  exact catalog and asserts no optional payload entered the installers.
