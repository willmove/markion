## Why

Markion already renders and source-maps rich Markdown, but three essential authoring paths still stop short of a dependable desktop-editor workflow: images cannot be ingested as managed document resources, visual formatting and links still expose placeholder-oriented source operations, and direct file writes cannot safely distinguish an ordinary save from an externally modified document. These are P0 because they affect data safety and the most common document-authoring operations.

## What Changes

- Add an end-to-end local image-resource workflow for clipboard images and dragged image files: choose a document-relative asset directory, copy or encode resources safely with collision-resistant names, insert portable relative Markdown image links, replace an existing image without losing its alt text or presentation metadata, expose practical size/alignment controls, and present an explicit missing-resource state.
- Add selection-contextual Visual Edit formatting controls for the existing source-backed Markdown commands, plus a visual link editor that edits label, URL, and optional title as one exact canonical-source mutation.
- Replace direct document writes with same-directory atomic replacement and durable cleanup behavior, while retaining the document path and dirty state when a write fails.
- Track the last known on-disk file identity, detect external changes before save and while the document is open, automatically reload only clean documents, and give dirty documents an explicit reload/overwrite/save-copy conflict choice.
- Strengthen recovery and session continuity so dirty named and untitled tabs have stable recovery snapshots, successful saves retire obsolete snapshots, and restart restoration never silently replaces newer disk content with recovery content.
- Add localized status, dialog, and accessibility text and executable model/GPUI/persistence regressions for these workflows.

Non-goals: remote image upload/credentials, a second rich-text document model, automatic rewriting of arbitrary external Markdown asset layouts, collaborative merge, or background cloud synchronization.

## Capabilities

### New Capabilities

- `document-resources`: Local image ingestion, relative asset links, replacement/presentation metadata, and missing-resource behavior.
- `reliable-file-persistence`: Atomic document writes, disk identity and conflict handling, and recovery snapshot lifecycle.

### Modified Capabilities

- `markdown-editing`: Visual Edit gains selection-contextual formatting and an exact source-backed link editor while preserving selection, undo, IME, and source mapping invariants.
- `workspace`: Open tabs and session restoration gain external-change observation and conflict-safe user-facing actions.
- `ui-i18n`: New resource, link-editor, external-change, and recovery UI text is localized across every supported language.

## Impact

The change primarily affects `MarkdownDocument`, the tab/session/recovery state, document save/open orchestration, GPUI clipboard/drop/input handling, Visual Edit rendering and overlays, image preview resolution, and localized UI strings. It adds no new canonical document representation and preserves per-version `Arc`-shared derived Markdown caches, memoized highlighting, cached text handles, exact UTF-8 source ranges, semantic undo grouping, and IME replacement paths. Any filesystem helper remains GPUI-free and reusable from tests.
