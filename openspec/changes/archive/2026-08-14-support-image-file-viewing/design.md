## Context

See `proposal.md` for motivation. The file tree currently classifies regular files as Markdown or curated text and skips every other file. All interactive open paths eventually call `MarkdownDocument::open`, so image bytes fail UTF-8 decoding. `MarkionApp` also stores `Vec<EditorTab>` and the root render path assumes the active tab always owns a Markdown document.

Markion already has the required image primitives: a shared supported-extension check for imported resources, SVG and raster decoding, asynchronous decode scheduling, a bounded `PreviewImageCache`, GPUI `RenderImage` presentation, and a file-image icon. The design must reuse those primitives, keep file-tree rendering bounded, and avoid invoking any Markdown-derived cache for an image.

The existing external-path drop behavior is intentionally different: dropping an image on an editable pane imports it as a document resource. The active `support-cli-open-paths` change separately owns startup/CLI path semantics.

## Goals / Non-Goals

**Goals:**

- Establish one supported-image classification used by the file tree and interactive open routing.
- Let text-document and read-only image tabs coexist behind a small common tab interface.
- Reuse the bounded preview-image decode/cache pipeline and release its claims with tab lifecycle changes.
- Keep the existing Markdown `EditorTab` state and all per-version derived caches intact.

**Non-Goals:**

- Turning binary assets into `MarkdownDocument` values or adding image-editing state.
- Adding zoom, pan, animation controls, image metadata inspection, or external-change watching for image tabs.
- Changing image drop/import behavior or CLI/startup path handling.
- Persisting image-tab decode results or adding dependencies.

## Decisions

### 1. Add a workspace-tab sum type around the existing document tab

Change the application tab vector to hold a wrapper such as:

```text
WorkspaceTab
├─ Document(EditorTab)      existing Markdown/text state, unchanged internally
└─ Image(ImageTabState)    path, cache key, and presentation/scroll state only
```

The wrapper will expose only content-independent queries such as `path`, `title`, `is_dirty`, and content kind. Document-only helpers will return an optional/reference-checked `EditorTab`; action handlers and render branches must first establish that the active content is a document. Close, focus, next/previous, title, current-tree selection, recent-file, and workspace-root logic will operate through the common path interface.

This keeps the large, performance-sensitive `EditorTab` and `MarkdownDocument` cache invariants localized. A dummy Markdown document containing image syntax was rejected because it would expose editing/save behavior, create false dirty/autosave state, and still run Markdown parsing. Adding optional image fields directly to `EditorTab` was rejected because it would permit invalid mixed states and spread `Option` checks through document internals.

### 2. Use one file-kind classifier and one interactive open router

Promote the existing case-insensitive image-extension set to the shared classifier used by resource import, file-tree scanning, icons, and interactive opening. Extend `FileTreeFileKind` with `Image`; directory filtering, hidden/noise filtering, sorting, filename filtering, row limits, and create/rename/delete behavior remain unchanged.

Interactive open entry points will call one router with an explicit intent (`ReplaceActive` or `OpenInNewTab`). The router will:

1. classify the path as document, image, or unsupported;
2. focus an existing filesystem-backed tab before loading anything;
3. open UTF-8 document content or construct an `ImageTabState` accordingly;
4. apply the existing dirty guard only when replacing a dirty document tab;
5. update workspace root, recent paths, selection, localized status, and cache claims through one success path.

File → Open uses `ReplaceActive`; file-tree clicks, Open in New Tab, and Open Recent use `OpenInNewTab`. Unsupported paths fail before any tab mutation. This removes the current duplicated `MarkdownDocument::open` branches and prevents file-tree/open-dialog support from drifting.

### 3. Branch rendering before Markdown derivation and reuse the image cache

The root render path will inspect the active `WorkspaceTab` before requesting preview blocks, visual blocks, outline, statistics, diagrams, math, or document text. The data flow is:

```text
supported local path
  → ImageTabState with explicit local PreviewImageKey
  → claim key in bounded PreviewImageCache
  → existing background read / decode / rasterize scheduler
  → Pending | Ready(RenderImage + presentation size) | Error
  → dedicated localized image-tab surface

tab becomes dormant, is replaced, or closes
  → release cache claim
  → existing byte/count LRU policy may retain or evict the decoded image
```

Add an explicit local-path key constructor instead of converting a Windows path through URL-oriented logic. The image surface will use the ready entry's logical presentation dimensions and compute a uniform scale of `min(1, available_width / width, available_height / height)`. It will center the result, preserve aspect ratio, reserve the normal content bounds, and keep an overflow container so constrained layouts remain usable. Pending and error entries render localized states; an error remains scoped to its tab.

Image tabs use the existing decoder limits, concurrency caps, maximum retained bytes, and raster downscaling. No second decoded-image store is introduced. Inactive image tabs release their claims just as dormant document tabs release preview-image claims; reactivation reclaims the key and reuses an LRU entry when available.

### 4. Make document-only actions explicit at the application boundary

Editing, selection, formatting, search/replace, save/autosave/recovery, outline, statistics, view-mode, and export handlers will guard on the active tab kind. Their existing document implementation remains unchanged. When an image is active, UI affordances that can be disabled will be disabled; shortcut-dispatched actions will leave state untouched and use a localized unavailable status where feedback is needed.

Dirty guards, quit checks, recovery cleanup, session document serialization, and external-document polling will iterate only document tabs. Image tabs never produce undo snapshots, dirty state, recovery files, or autosave timers. Replacing or closing a document still releases all of its preview-image claims; replacing or closing an image releases its single viewer claim.

### 5. Verify classification, routing, lifecycle, and rendering seams separately

Storage tests will cover every supported extension (including mixed case), `FileTreeFileKind::Image`, hidden/noise behavior, folder structure, and exclusion of unsupported binaries. Application tests will cover each interactive open intent, path de-duplication across content kinds, dirty-guard behavior, close/quit/autosave scoping, document-state preservation across image activation, cache claim release, and localized failure rendering. GPUI render tests will assert the image branch bypasses document panes and derives a centered, aspect-preserving fit without upscaling.

## Risks / Trade-offs

- **[Risk] Converting `tabs` to a sum type touches many document access sites.** → Keep the existing `EditorTab` intact, introduce narrow document/image accessors, migrate callers by subsystem, and use compiler errors plus document regression tests to find unguarded assumptions.
- **[Risk] Large or malformed images could consume memory or crash the UI.** → Reuse the established background decoder, maximum-edge rasterization, concurrency limits, bounded byte/count cache, and error entries; never decode in the frame path.
- **[Risk] Supported-extension lists could diverge between import, tree icons, scanning, and opening.** → Define and test one case-insensitive classifier and make all four callers depend on it.
- **[Risk] Global menus and shortcuts may still dispatch while an image is active.** → Guard every document-only action at the common application boundary and test representative menu, shortcut, autosave, and quit flows.
- **[Trade-off] Image tabs do not watch or automatically reload external file changes.** → Keep this change focused on opening and presentation; closing/reopening after cache eviction reloads from disk, while explicit refresh/reload semantics can be specified separately.

## Migration Plan

No persisted data migration is required. Implement the tab wrapper and compatibility accessors first, then add classification/routing and the image render branch, and finally enable file-tree exposure. The change can be rolled back by reverting those code changes; Markdown documents, preferences, workspaces, and files on disk remain compatible because no storage format changes.
