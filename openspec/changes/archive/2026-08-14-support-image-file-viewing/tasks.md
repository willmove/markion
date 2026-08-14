## 1. Shared Image Classification and File Tree

- [x] 1.1 Make the supported image-extension set a single case-insensitive storage classifier shared by resource import, interactive opening, file-tree scanning, and file icons; add unit coverage for every supported extension, mixed-case paths, and unsupported files.
- [x] 1.2 Add `FileTreeFileKind::Image`, include supported images in background workspace scans without changing ignored/hidden-directory or bounded-row behavior, and update scan/filter/hidden/folder tests to cover image entries.
- [x] 1.3 Render every supported image entry with the existing file-image icon and preserve the existing selection, collapse, context-menu, rename, delete, refresh, and filename-filter behavior.

## 2. Heterogeneous Tab Model

- [x] 2.1 Introduce a `WorkspaceTab` document/image sum type and an `ImageTabState` containing only path, cache key, and presentation state; keep the existing document `EditorTab` internals and cached-per-version Markdown state unchanged.
- [x] 2.2 Add common tab helpers for path, title, dirty state, focus identity, and tab navigation, then migrate generic tab-bar, current-tree marking, workspace-root, recent-file, close, and last-tab behavior to those helpers.
- [x] 2.3 Restrict dirty guards, quit checks, autosave/recovery cleanup and scheduling, session document serialization, external-file observation, and document image-claim management to document tabs; add regression tests proving image tabs never become dirty, autosaved, recovered, or counted as unsaved.
- [x] 2.4 Add image-tab cache-claim lifecycle handling for activation, dormancy, replacement, and close, including tests that claims are released without evicting or corrupting another tab's shared preview image.

## 3. Interactive Open Routing

- [x] 3.1 Implement one supported-path router with replace-active and open-in-new-tab intents, existing-path de-duplication, document UTF-8 loading, image-tab construction, and non-destructive rejection of unsupported files.
- [x] 3.2 Route File → Open through replace-active semantics, applying the existing dirty guard only to editable document tabs and allowing an image tab to be replaced without an unsaved-changes prompt.
- [x] 3.3 Route Open in New Tab, file-tree clicks/context actions, and Open Recent through new-tab semantics, and verify image success updates workspace root, recent paths, localized status, and current-file marking exactly once.
- [x] 3.4 Preserve OS image-drop import behavior and the separate CLI/startup path gate; add regression tests showing neither path is silently changed by the interactive router.

## 4. Image Loading and Presentation

- [x] 4.1 Add an explicit local-path `PreviewImageKey` constructor and an image-tab load request that reuses the existing background decoder, concurrency caps, maximum-edge rasterization, and bounded byte/count cache.
- [x] 4.2 Branch the root workspace render before any Markdown preview, visual, outline, statistics, diagram, math, or editor derivation when an image tab is active, and add a regression test proving document versions and derived caches survive switching through an image tab.
- [x] 4.3 Render localized loading and unavailable-image states plus a dedicated ready-image surface that centers the decoded frame, preserves aspect ratio, fits oversized images in both dimensions, and never upscales smaller images; unit-test the fit calculation and GPUI-render the ready/error branches.
- [x] 4.4 Present animated sources as a static decoded frame and verify corrupt, missing, SVG, raster, large, and mixed-case-extension fixtures remain contained, closable, and responsive.

## 5. Document-Only Commands and Localization

- [x] 5.1 Guard editing, selection, formatting, search/replace, save, outline, statistics, view-mode, and export actions at the application boundary so they cannot read document state or mutate files while an image tab is active.
- [x] 5.2 Disable document-only menu/sidebar affordances where practical and add localized loading, unavailable-image, unsupported-file, and action-unavailable text in every supported language.
- [x] 5.3 Add application tests covering shortcuts and menus on image tabs, no close confirmation for images, correct dirty confirmation for documents replaced by images, duplicate-path focusing, and restoration of document selection/history/scroll state.

## 6. Verification

- [x] 6.1 Run formatting and focused storage/application/render tests, then resolve all failures without weakening the image-viewing or Markdown cache invariants.
- [x] 6.2 Run `cargo test --workspace` and `openspec validate support-image-file-viewing`, and manually smoke-test File → Open, Open in New Tab, Open Recent, and file-tree opening for successful and corrupt images on the current platform.
