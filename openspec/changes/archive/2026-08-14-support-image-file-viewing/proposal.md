## Why

Image assets used by Markdown documents are currently hidden from the workspace tree and opening one attempts to decode it as UTF-8 text. Users therefore have to leave Markion to inspect an image that is already part of the document workspace.

## What Changes

- List supported local image files (`png`, `jpg`/`jpeg`, `gif`, `webp`, `bmp`, `tif`/`tiff`, and `svg`, case-insensitively) alongside Markdown and curated text files in the Files sidebar.
- Open supported image paths from File → Open, Open in New Tab, Open Recent, and the file tree as read-only image tabs, while focusing an existing tab for the same path instead of duplicating it.
- Present the decoded image on a dedicated, scrollable, centered surface that fits oversized images within the available content area without upscaling smaller images.
- Show a localized, non-destructive error state when an image cannot be read or decoded.
- Keep image tabs outside Markdown editing, parsing, autosave, recovery, outline, formatting, and export behavior.

Non-goals: editing images, animated-image playback, remote-image browsing, changing the existing drop-to-import workflow, or extending the separate CLI/startup-path change.

## Capabilities

### New Capabilities

- `image-file-viewing`: Read-only opening, tab identity, presentation, and failure behavior for supported local image files.

### Modified Capabilities

- `workspace`: Extend file-tree discovery, filtering, selection, and opening to supported image files.
- `markdown-editing`: Generalize the tab host so read-only image tabs can coexist with Markdown/text tabs without weakening per-document editing-state isolation.

## Impact

- Affects file classification and scanning in `src/storage/file_tree.rs`, tab/content state and open-path routing in `src/app/`, root workspace rendering, recent-file handling, and localized strings in `src/i18n.rs`.
- Reuses the existing bounded preview-image decode/cache path and file-image icon rather than adding another decoder or GPUI image representation; image-tab lifecycle must claim and release cached images so memory remains bounded.
- Preserves bounded file-tree row rendering and the per-version derived Markdown caches; activating an image tab must not parse Markdown or invalidate a text tab's cached state.
- No new external dependency or persisted-file format is required.
