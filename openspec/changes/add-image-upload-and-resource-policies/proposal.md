## Why

Markion already imports pasted and dropped images into a fixed `<document>.assets/` directory, but authors cannot choose cloud hosting or a different local resource layout. A unified image workflow will support both new insertions and existing documents, making notes portable for offline use or publication without manually moving files and rewriting image links.

## What Changes

- Add independent insertion policies for local files (keep reference, copy, upload), clipboard image bytes (save locally, upload), and remote images (keep URL, download locally, download then upload). Existing defaults remain local storage for files/clipboard images and unchanged URLs for remote images.
- Add document-relative resource directory templates, including the existing `{document}.assets`, `assets`, and `assets/{document}`; compute complete relative URLs for nested directories and preserve collision-safe storage and reuse.
- Resolve encoded local URLs consistently in preview, including Chinese document names, and contain concise image-load failure placeholders within the document column.
- Add upload adapters for PicGo's local HTTP server, PicGo Core CLI, and user-configured commands. Aliyun OSS and other hosts are configured in PicGo or the user's script; Markion accepts the resulting image URL.
- Add an Images preferences tab for policies, directory previews, uploader configuration, explicit test uploads, and a visible scrollbar for the full settings pane. Persist settings in optional TOML sections with compatible defaults.
- Route file/URL insertion, clipboard images, image references in pasted Markdown/rich text, drops, and image replacement through the same policy pipeline. Keep ordinary typing, opening, previewing, and saving free of automatic image transfers.
- Add single-image Upload / Save to resource directory actions and current-document batch actions for uploading local/embedded images, downloading remote images, and organizing local/embedded images. Group Format-menu image commands under one Images submenu, and place selected-image presentation, source, replacement, save, and upload commands in the visual editor's right-click menu. Batch review reports eligible, repeated, missing, and unsupported references before execution.
- Run transfers off the UI thread, retain recoverable inputs/results on failure, and apply successful destination changes only to verified original targets. Preserve authored image metadata, provide cancellation/retry and partial-result reporting, and make each completed replacement batch one undoable edit.

**Non-goals:** native OSS/S3 credential management or SDK integrations, installing/managing PicGo, deleting cloud objects on undo, background folder synchronization, workspace-wide migration, automatic orphan cleanup, and automatic cloud uploads during DOCX import or WeChat export.

## Capabilities

### New Capabilities

- `image-uploading`: PicGo HTTP, PicGo Core CLI, custom-command upload contracts, bounded transfer execution, result validation, cancellation, and diagnostic behavior.

### Modified Capabilities

- `document-resources`: configurable insertion policies and local directories, single-image and current-document transformations, precise asynchronous destination replacement, and recoverable failures.
- `theme-preferences`: Images tab with editable policy, directory, uploader, and upload-test controls plus preference persistence/reset behavior.
- `chrome-platform`: extend the persisted preference scope to image handling and external-uploader connection settings while keeping cloud credentials owned by the external tool.

## Impact

- Application integration: `src/app/editing.rs`, `workspace.rs`, `publishing.rs`, `root_view.rs`, `bootstrap.rs`, `appearance.rs`/`search.rs`, and app state/action definitions. The existing syntax insertion remains available alongside explicit file/URL insertion.
- Domain/storage: `src/model.rs`, `src/storage/preferences.rs`, `src/storage/resources.rs`, `src/lib.rs` image destination scanning/mutation helpers, and `src/inline_edit.rs`; extend existing exact source mapping instead of global text replacement.
- Background execution: reuse the existing HTTP runtime in `src/app/network.rs`; add a dedicated bounded image-transfer module and process adapter. Reuse installed dependencies where suitable; no cloud-vendor SDK or bundled Node runtime is required.
- Rich paste: integrate retained image references from `crates/html-import` conversion without adding networking or GPUI to that crate. Embedded images already present in Markdown/raw HTML are eligible for explicit resource actions; expanding the HTML converter's currently discarded data-image input is deferred.
- Existing resource organization, Git write admission, image cache invalidation, DOCX import's transactional resource publication, and local publishing resolution require compatibility coverage. Resource settings affect future explicit operations, not historical links or asset directories.
- All new user-visible strings use `src/i18n.rs`. Background progress must not mutate document versions; only accepted source edits invalidate the existing per-version `Arc` caches. No network/process work or document-wide scans enter the rendering or per-keystroke derivation path.
- Add unit, fake-adapter, loopback HTTP, and GPUI regression coverage, plus user documentation for PicGo/OSS setup, command output, directory templates, and recovery. Cross-platform process execution and packaged-app smoke tests are required before calling implementation complete.
